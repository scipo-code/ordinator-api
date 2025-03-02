import { http } from "msw";

const resourceData = {
  "assets": [
    {
      "asset": "DF",
      "metadata": {
        "periods": [
          { "id": "Week1-2", "label": "Week 1-2" },
          { "id": "Week3-4", "label": "Week 3-4" },
          { "id": "Week5-6", "label": "Week 5-6" },
          { "id": "Week7-8", "label": "Week 7-8" },
        ],
        "resources": [
          { "id": "MTN-ELEC", "label": "MTN-ELEC" },
          { "id": "MTN-INST", "label": "MTN-INST" },
          { "id": "MTN-MECH", "label": "MTN-MECH" },
          { "id": "MTN-CRAN", "label": "MTN-CRAN" },
          { "id": "MTN-ROUS", "label": "MTN-ROUS" },
        ],
      },
      "data": [
        {
          "periodId": "Week1-2",
          "values": {
            "MTN-ELEC": 2,
            "MTN-INST": 2,
            "MTN-MECH": 2,
            "MTN-CRAN": 1,
            "MTN-ROUS": 4,
          },
        },
        {
          "periodId": "Week3-4",
          "values": {
            "MTN-ELEC": 3,
            "MTN-INST": 2,
            "MTN-MECH": 2,
            "MTN-CRAN": 1,
            "MTN-ROUS": 5,
          },
        },
        {
          "periodId": "Week5-6",
          "values": {
            "MTN-ELEC": 2,
            "MTN-INST": 3,
            "MTN-MECH": 3,
            "MTN-CRAN": 2,
            "MTN-ROUS": 4,
          },
        },
        {
          "periodId": "Week7-8",
          "values": {
            "MTN-ELEC": 4,
            "MTN-INST": 3,
            "MTN-MECH": 2,
            "MTN-CRAN": 2,
            "MTN-ROUS": 6,
          },
        },
      ],
    },
    {
      "asset": "TL",
      "metadata": {
        "periods": [
          { "id": "Week1-2", "label": "Week 1-2" },
          { "id": "Week3-4", "label": "Week 3-4" },
          { "id": "Week5-6", "label": "Week 5-6" },
          { "id": "Week7-8", "label": "Week 7-8" },
        ],
        "resources": [
          { "id": "MTN-TELE", "label": "MTN-TELE" },
          { "id": "MTN-TURB", "label": "MTN-TURB" },
        ],
      },
      "data": [
        {
          "periodId": "Week1-2",
          "values": {
            "MTN-TELE": 1,
            "MTN-TURB": 1,
          },
        },
        {
          "periodId": "Week3-4",
          "values": {
            "MTN-TELE": 2,
            "MTN-TURB": 2,
          },
        },
        {
          "periodId": "Week5-6",
          "values": {
            "MTN-TELE": 1,
            "MTN-TURB": 2,
          },
        },
        {
          "periodId": "Week7-8",
          "values": {
            "MTN-TELE": 2,
            "MTN-TURB": 3,
          },
        },
      ],
    },
  ],
};

export const handlers = [
  http.get("/api/scheduler/:asset/resources", ({ params }) => {
    const { asset } = params;
    // Filter json based on asset
    const assetResources = resourceData.assets.find((a) =>
      a.asset.toLowerCase() === asset
    );

    if (!assetResources) {
      return new Response("Asset not found", { status: 404 });
    }

    return Response.json(assetResources, { status: 200 });
  }),

  http.get(
    "/api/scheduler/:asset/resources/:periodId",
    ({ params }: { params: { asset: string; periodId: string } }) => {
      const { asset, periodId } = params;
      console.log(asset, periodId);
      // Filter json based on asset
      const assetResources = resourceData.assets.find((a) =>
        a.asset.toLowerCase() === asset
      );

      if (!assetResources) {
        return Response.json({ error: "Asset not found" }, { status: 404 });
      }

      const periodResources = assetResources.data.find(
        (d) => d.periodId.toLowerCase() === periodId.toLowerCase(),
      );

      if (!periodResources) {
        return Response.json({ error: "Period not found" }, {
          status: 404,
        });
      }

      const filteredMetadataPeriods = assetResources.metadata.periods.filter(
        (p) => p.id.toLowerCase() === periodId.toLowerCase(),
      );
      const filteredMetadataResources = assetResources.metadata.resources
        .filter(
          (r) => r.id.toLowerCase() === periodId.toLowerCase(),
        );

      const periodResponse = {
        asset: assetResources.asset,
        metadata: {
          periods: filteredMetadataPeriods,
          resources: filteredMetadataResources,
        },
        data: [periodResources],
      };
      return Response.json(periodResponse, { status: 200 });
    },
  ),

  http.put(
    "/api/scheduler/:asset/resources/:periodId",
    async ({ request, params }) => {
      const { asset, periodId } = params;
      const updatedPeriod = await request.json();
      console.log(
        'Updating asset "%s" for period "%s" with:',
        asset,
        periodId,
        updatedPeriod,
      );

      return Response.json({ status: 200 });
    },
  ),
];
