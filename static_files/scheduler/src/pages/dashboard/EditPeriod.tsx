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
import { useEffect, useState } from "react";
import { AssetResourceApiResponse } from "@/types";


interface EditPeriodDialogProps {
  asset: string,
  periodId: string,
  onClose: () => void,
};





const EditPeriodDialog: React.FC<EditPeriodDialogProps> = ({ asset, periodId, onClose }) => {
  const [downloading, setDownloading] = useState(true);
  const [formValues, setFormValues] = useState<Record<string, number | string>>({})
  const [error, setError] = useState<null | string>(null);

  useEffect(() => {
    const fetchPeriod = async () => {
      setError(null);

      try {
        const { data } = await axios.get<AssetResourceApiResponse>(`/api/scheduler/${asset}/resources/${periodId}`);

        if (data.data[0].periodId.toLowerCase() !== periodId.toLowerCase()) {
          throw new Error("Endpoint returned wrong periods")
        } else {
          setFormValues(data.data[0].values);
        }
        
      } catch (error) {
        setError(`Error fetching period: ${periodId}`);
        console.error(`Error fetching period: ${periodId}`, error);
      } finally {
        setDownloading(false);
      }
    };

    fetchPeriod()
    
  }, [asset, periodId])

  const handleInputChange = (resourceId: string, value: string) => {
    setFormValues((prev) => ({...prev, [resourceId]: value }))
  }

  const uploadResources =  async () => {
    console.log("uploading")
  }

  return (
        <Dialog open onOpenChange={(open) => !open && onClose()}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Update period resources</DialogTitle>
              <DialogDescription>
                {periodId}
              </DialogDescription>
            </DialogHeader>
          <div className="grid gap-4 py-4">
            {Object.keys(formValues).map((resourceId) => (
              <div key={resourceId} className="grid grid-cols-4 items-center gap-4">
                <Label htmlFor={resourceId} className="text-right">
                  {resourceId}
                </Label>
                <Input
                  id={resourceId}
                  type="number"
                  min={0}
                  value={formValues[resourceId]}
                  onChange={(e) => handleInputChange(resourceId, e.target.value)}
                  className="col-span-3"
                />
              </div>
            ))}
          </div>
            { error ? (
              <p className="text-red-600">{error}</p>
            ) : null}
          <DialogFooter>
            <Button onClick={uploadResources} disabled={downloading} type="submit">
            {downloading ? (
              <>
                <ReloadIcon className="mr-2 h-4 w-4 animate-spin" />
                Please wait
              </>
            ) : (
              'Update'
            )}
              </Button>
          </DialogFooter>
          </DialogContent>
        </Dialog>
  )
};

export default EditPeriodDialog;
