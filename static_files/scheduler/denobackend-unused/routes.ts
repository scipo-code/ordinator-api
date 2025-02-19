import { Router } from "@oak/oak";
import "jsr:@std/dotenv/load";
import assets from "./assets.json" with { type: "json" };
import resources from "./resources.json" with { type: "json" };

const router = new Router();

console.log("Starting the ordinator-scheduler-frontend");

router.get("/api/scheduler/assets", (context) => {
  context.response.body = assets;
});

router.get("/api/scheduler/export/:asset", (context) => {
  let asset = context?.params?.asset;
  context.response.body = asset;
});

router.get("/api/resources/:asset", (context) => {
  context.response.body = resources;
});

router.get("/api/scheduler/export/:asset", async (context) => {
  let asset = context?.params?.asset;

  if (!asset) {
    context.response.status = 400;
    context.response.body = "No asset provided.";
    return;
  }

  const isValidAsset = assets.assets.find((item) => item.value === asset);
  console.log(isValidAsset);

  if (!isValidAsset) {
    context.response.status = 400;
    context.response.body = "Invalid asset provided.";
    return;
  }

  try {
    let command;
    if (Deno.build.os == "windows") {
      command = Deno.env.get("IMPERIUM_PATH_WINDOWS");
    } else if (Deno.build.os == "linux") {
      command = Deno.env.get("IMPERIUM_PATH_LINUX");
    } else {
      context.response.status = 500;
      context.response.body = "IMPERIUM_PATH_<PLATFORM> variable not found";
      return;
    }

    if (!command) {
      context.response.status = 500;
      context.response.body = "exec error: didn't find command";
      return;
    }

    const subcommand = "export";
    asset = asset.toLowerCase();
    console.log(`Calling ${command} ${subcommand} ${asset} ...`);
    const cmd = new Deno.Command(command, {
      args: [
        subcommand,
        asset,
      ],
      stdout: "piped",
      stderr: "piped",
    });

    // stdout here returns the JSONs that should be handled by the frontend
    const { code, stdout, stderr } = await cmd.output();

    if (code !== 0) {
      console.log(
        `Error executing command: ${new TextDecoder().decode(stderr)}`,
      );
      context.response.status = 500;
      context.response.body = "exec error";
      return;
    }

    const excelBytes = await Deno.readFile("ordinator_dump.xlsx");

    context.response.headers.set(
      "Content-Type",
      "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
    );
    context.response.headers.set(
      "Content-Disposition",
      'attachment; filename="scheduling.xlsx"',
    );
    context.response.headers.set(
      "Content-Length",
      excelBytes.length.toString(),
    );

    context.response.body = new ReadableStream({
      start(controller) {
        controller.enqueue(excelBytes);
        controller.close();
      },
    });
  } catch (error) {
    console.error("Unexpected error", error);
    context.response.status = 500;
    context.response.body = "Unexpected server error";
  }
});

export default router;
