import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import axios from "axios";
import { ReloadIcon } from "@radix-ui/react-icons";
import { useState } from "react";


interface ExportDialogProps {
  asset: string,
};





const ExportDialog: React.FC<ExportDialogProps> = ({ asset }) => {
  const [downloading, setDownloading] = useState(false);
  const [error, setError] = useState<null | string>(null);

  const currentDate = new Date().toISOString().slice(0,10);
  const defaultFilename = `${currentDate}_schedule.xlsx`;
  
  const [filename, setFilename] = useState(defaultFilename);
  const downloadExcel = async () => {
    setError(null);
    setDownloading(true);

    try {
      const response = await axios.get(`/api/scheduler/export/${asset}`,  { responseType: 'blob' });
      console.debug(response);
  
      if (response.status !== 200) {
        throw new Error('Network response was not ok');
      }

      const contentType = response.headers["Content-Type"];
      const expectedType = 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet';
      if (contentType !== expectedType) {
        throw new Error("Received file was not a valid excelfile");
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

    } catch (error) {
      setError("Error downloading file");
      console.error('Error downloading file', error);
    } finally {
      setDownloading(false);
    }
  };

  return (
        <Dialog>
          <DialogTrigger asChild>
            <Button variant="default">Export</Button>
            </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Export current workorders to Excel</DialogTitle>
              <DialogDescription>
                This will export the current workorders to excel. Working in excel disconnects changes from the scheduling system.
              </DialogDescription>
            </DialogHeader>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="filename" className="text-right">
                Filename
              </Label>
              <Input id="filename" value={filename} onChange={(e) => setFilename(e.target.value)} className="col-span-3" />
            </div>
          </div>
        { error ? (
          <p className="text-red-600">{error}</p>
        ) : null}
          <DialogFooter>
            <Button onClick={downloadExcel} disabled={downloading} type="submit">
            {downloading ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Please wait
              </>
            ) : (
              'Download'
            )}
              </Button>
          </DialogFooter>
          </DialogContent>
        </Dialog>
    
  )
};

export default ExportDialog;
