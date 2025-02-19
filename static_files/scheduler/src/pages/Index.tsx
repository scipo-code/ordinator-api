import axios from 'axios';
import { Asset } from '../types.ts';
import { useEffect, useState } from 'react';
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { Button } from '@/components/ui/button.tsx';
import { ReloadIcon } from '@radix-ui/react-icons';



export default function Index() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [selectedAsset, setSelectedAsset] = useState('');
  const [downloading, setDownloading] = useState(false);
  // const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const fetchAssets = async () => {
      try {
        const response = await axios.get('api/scheduler/assets');
        const assetsData = response.data;
        
        console.debug("Retrived assets");
        setAssets(assetsData);
      } catch (error) {
        // setError('Error fetching assets');
        setAssets([]);
        console.error("Could not fetch assets {error}");
      }
    };

    fetchAssets();
    
  }, []);

  const downloadExcel = async () => {
    setDownloading(true);

    try {
      const response = await axios.get(`/api/scheduler/export/${selectedAsset}`,  { responseType: 'blob' });
      console.debug(response);
  
      if (response.status !== 200) {
        throw new Error('Network response was not ok');
      }

      console.log(response.data);
      const blob = new Blob([response.data], { type: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'});
      const url = window.URL.createObjectURL(blob);

      const link = document.createElement('a');
      link.href = url;
      link.setAttribute('download', 'scheduling.xlsx');
      document.body.appendChild(link);

      link.click();

      link.parentNode?.removeChild(link);
      window.URL.revokeObjectURL(url);

      setDownloading(false);
    }  catch (error) {
      console.error('Error downloading file', error);
      setDownloading(false);
    }
  };

  const handleAssetChange = (value: string) => {
    setSelectedAsset(value);
    console.log(value);
  };

  
  return (
    <main>
      <div className="flex justify-center">
        <Card className="w-[380px]">
          <CardHeader className="text-left">
            <CardTitle>Workorder Schedules</CardTitle>
            <CardDescription>Download a spreadsheet with the currently scheduled workorders</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-4">
            <p className="text-left font-medium">Asset</p>
            <Select onValueChange={handleAssetChange}>
              <SelectTrigger>
                <SelectValue placeholder="Select an asset" />
              </SelectTrigger>
              <SelectContent>
                
                {
                assets && assets.length > 0 ? (
                  assets.map((asset: Asset) => {
                    return(
                      <SelectItem key={asset.value} value={asset.value}>
                        {asset.label}
                      </SelectItem>
                    );
                  })
                ) : (
                  <p> Could not fetch assets </p>
                )
               }
            </SelectContent>
          </Select>
          </CardContent>
          <CardFooter>
            <Button 
              variant={selectedAsset === '' ? 'secondary' : 'default'}
              disabled={selectedAsset === '' || downloading}
              onClick={downloadExcel}
            >
            {downloading ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Please wait
              </>
            ) : (
              'Download'
            )}
            </Button>
            
          </CardFooter>
        </Card>
      </div>
    </main>
  )
}
