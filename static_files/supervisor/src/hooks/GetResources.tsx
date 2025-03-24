import { AssetResourceApiResponse } from "@/types";
import { useQuery } from "@tanstack/react-query";
import axios from "axios";

// TODO: The response should be checked for it to be a json

export function getResources(asset: string) {
  return useQuery<AssetResourceApiResponse>({
    queryKey: ["resources", asset],
    queryFn: async () => {
      const res = await axios.get<AssetResourceApiResponse>(`api/v1/${asset}/resources`);

      if (res.status !== 200) {
        throw new Error(`request failed with status ${res.status}`)
      }

      console.log(res.data)

      return res.data
      }
    }
  )
  
}
