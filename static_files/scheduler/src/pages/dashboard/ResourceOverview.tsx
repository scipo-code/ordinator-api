// import axios from "axios";
import { useState, useEffect } from "react";
import { Container } from "@/components/Container";
// import { useParams } from "react-router-dom";
import { ResourcePeriod, ResourceTableRow } from "@/types";
import { DataTable } from "./data-table";
import { transformResourcePeriodToResourceRow } from "./utils";
import { createColumnsFromResourceData } from "./columns";
import { ColumnDef } from "@tanstack/react-table";
// import { columns } from "./columns"; 

const initialResources =  [
  {
    "period": "Week1-2",
    "workCenters": [
      { "workCenter": "MTN-ELEC", "noOfTech": 2 },
      { "workCenter": "MTN-INST", "noOfTech": 2 },
      { "workCenter": "MTN-MECH", "noOfTech": 2 },
      { "workCenter": "MTN-TELE", "noOfTech": 1 },
      { "workCenter": "MTN-CRAN", "noOfTech": 1 },
      { "workCenter": "MTN-TURB", "noOfTech": 1 },
      { "workCenter": "MTN-ROUS", "noOfTech": 4 }
    ]
  },
  {
    "period": "Week3-4",
    "workCenters": [
      { "workCenter": "MTN-ELEC", "noOfTech": 3 },
      { "workCenter": "MTN-INST", "noOfTech": 2 },
      { "workCenter": "MTN-MECH", "noOfTech": 2 },
      { "workCenter": "MTN-TELE", "noOfTech": 2 },
      { "workCenter": "MTN-CRAN", "noOfTech": 1 },
      { "workCenter": "MTN-TURB", "noOfTech": 2 },
      { "workCenter": "MTN-ROUS", "noOfTech": 5 }
    ]
  },
  {
    "period": "Week5-6",
    "workCenters": [
      { "workCenter": "MTN-ELEC", "noOfTech": 2 },
      { "workCenter": "MTN-INST", "noOfTech": 3 },
      { "workCenter": "MTN-MECH", "noOfTech": 3 },
      { "workCenter": "MTN-TELE", "noOfTech": 1 },
      { "workCenter": "MTN-CRAN", "noOfTech": 2 },
      { "workCenter": "MTN-TURB", "noOfTech": 2 },
      { "workCenter": "MTN-ROUS", "noOfTech": 4 }
    ]
  },
  {
    "period": "Week7-8",
    "workCenters": [
      { "workCenter": "MTN-ELEC", "noOfTech": 4 },
      { "workCenter": "MTN-INST", "noOfTech": 3 },
      { "workCenter": "MTN-MECH", "noOfTech": 2 },
      { "workCenter": "MTN-TELE", "noOfTech": 2 },
      { "workCenter": "MTN-CRAN", "noOfTech": 2 },
      { "workCenter": "MTN-TURB", "noOfTech": 3 },
      { "workCenter": "MTN-ROUS", "noOfTech": 6 }
    ]
  }
];

export default function ResourceOverview() {
  // TODO: Use url to fetch
  // const { asset } = useParams<{ asset: string}>();

  const [ resources, setResources ] = useState<ResourcePeriod[]>([])
  const [ tableData, setTableData ] = useState<ResourceTableRow[]>([]);
  const [ columns, setColumns ] = useState<ColumnDef<ResourceTableRow>[]>([]);

  // TODO: Fetch resources from data
  // useEffect(() => {
  //   const fetchResources = async () => {
  //     try {
  //       const response = await axios.get(`/api-bridge/resources/${asset}`);
  //       const resourceData = response.data.resources;
  //       console.log("Resource data: ", resourceData);

  //       setResources(resourceData);
  //     } catch (error) {
  //       console.error(error);
  //     }
  //   };

  //   fetchResources();
  // }, [asset]);
  //

  useEffect(() => {
    setResources(initialResources);
  }, [])

  useEffect(() => {
    if (resources.length > 0) {
      const tableData = transformResourcePeriodToResourceRow(resources);
      const columns = createColumnsFromResourceData(tableData);

      console.log("td: ", tableData);
      console.log("cols: ", columns);

      setTableData(tableData);
      setColumns(columns);
    }
  }, [resources])


  return (
    <Container maxWidth="full" padding="sm" className="bg-white border border-gray-300 shadow rounded-lg">
      <DataTable columns={columns} data={tableData} />
    </Container>
  );
}
