import { Application} from "@oak/oak";
import { oakCors } from "@tajpouria/cors";
import "jsr:@std/dotenv/load";
import routeStaticFilesFrom from "./util/routeStaticFilesFrom.ts";
import router from "./routes.ts";

console.log("Starting the ordinator-scheduler-frontend");


const app = new Application();
app.use(oakCors());
app.use(router.routes());
app.use(router.allowedMethods());

const serveStaticFiles = Deno.env.get("SERVE")?.toLowerCase();
console.log("env variable 'SERVE' set to: ", serveStaticFiles);
if (serveStaticFiles == "true") {
  console.log("Frontend served from port: ", Deno.env.get("API_BRIDGE_PORT"));

  // Middleware that serves the static files in the dist and public folder. 
  // WARNING: It is important that `dist` remains the first folder in this list
  app.use(routeStaticFilesFrom([
    `${Deno.cwd()}/dist`,
    `${Deno.cwd()}/public`,
  ]));
} else {
  console.log("Frontend files served from vite server on port: 5173");
}

await app.listen({
  hostname: Deno.env.get("API_BRIDGE_HOST"),
  port: parseInt(
    Deno.env.get("API_BRIDGE_PORT") ||
      "You need to set a port for the application to run",
    10,
  ),
});
