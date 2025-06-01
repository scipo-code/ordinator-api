import { AssetResourceApiResponse } from "@/types";
import { useQuery } from "@tanstack/react-query";
import axios from "axios";



// TODO: The response should be checked for it to be a json
export function getResources(asset: string) {
  console.log("GetResources: line 9")
  return useQuery<AssetResourceApiResponse>({
    queryKey: [ asset],
    queryFn: async () => {
      `api/v1/supervisor/${asset}/resources`
      const res = await axios.get<AssetResourceApiResponse>(`api/v1/supervisor/${asset}/resources`);

      console.log(res)
      if (res.status !== 200) {

        throw new Error(`request failed with status ${res.status}`)
      }

      console.log(res.data)

      return res.data
      }
    }
  )
  
}
