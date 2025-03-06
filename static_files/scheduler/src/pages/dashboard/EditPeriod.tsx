import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import axios from "axios";
import { ReloadIcon } from "@radix-ui/react-icons";
import { useEffect, useState } from "react";
import { AssetResourceApiResponse } from "@/types";
import { useQuery } from "@tanstack/react-query";


interface EditPeriodDialogProps {
  asset: string,
  periodId: string,
  onClose: () => void,
  onUpdate: () => void,
};



const EditPeriodDialog: React.FC<EditPeriodDialogProps> = ({ asset, periodId, onClose, onUpdate }) => {
  const [formValues, setFormValues] = useState<Record<string, number | string>>({})
  const [uploading, setUploading] = useState(false);

  const {
    data,
    error,
    isLoading,
  } = useQuery<AssetResourceApiResponse>({
    queryKey: ["resources", asset, periodId],
    queryFn: async () => {
      const res = await axios.get<AssetResourceApiResponse>(`/api/scheduler/${asset}/resources/${periodId}`);

      if (res.status !== 200) {
        throw new Error(`Request failed with status: ${res.status}`)
      }

      return res.data
    }
  });

  useEffect(() => {
    if (!isLoading && !error && data) {
      if (data.data.length === 1) {
        setFormValues(data.data[0].values)
        console.log(formValues);
      }
    }
  }, [isLoading, error, data]);

  
  const handleInputChange = (resourceId: string, value: string) => {
    setFormValues((prev) => ({...prev, [resourceId]: value }))
  }

  const uploadResoucesForPeriod = async () => {
    setUploading(true);
    const payload: AssetResourceApiResponse = {
      asset,
      metadata: {
        periods: [{id: periodId, label: periodId}],
        resources: Object.keys(formValues).map((resourceId) => ({
          id: resourceId,
          label: resourceId
        })),
      },
      data: [
        {
          periodId,
          values: Object.keys(formValues).reduce((acc, resourceId) => {
            acc[resourceId] = Number(formValues[resourceId]);
            return acc;
          }, {} as Record<string, number>),
        },
      ],
    };

    console.log("From Client: ", payload);

    try {
      const response = await axios.put(
        `api/scheduler/${asset}/resources/${periodId}`,
        payload,
      );
      console.log("Client got response: ", response.data)
      // This rerenders the table
      onUpdate();

      // This closes the modal
      onClose();
    } catch (error) {
      setUploading(false);
      console.error("Error updating resources: ", error);
    }
  };

  return (
        <Dialog open onOpenChange={(open) => !open && onClose()}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Update period resources</DialogTitle>
              <DialogDescription>
                {periodId}
              </DialogDescription>
            </DialogHeader>
          { isLoading && (
            <div className="p-4">Loading period data...</div>
          )}

          { error && (
            <p className="p-4 text-red-600">
              {(error as Error).message || "Error fetching period"}
            </p>
          )}

          {!isLoading && !error && (
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
            
          )}
          <DialogFooter>
            <Button onClick={uploadResoucesForPeriod} disabled={uploading || isLoading} type="submit">
            {uploading ? (
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
