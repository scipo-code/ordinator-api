import { Next } from "jsr:@oak/oak/middleware";
import { Context } from "jsr:@oak/oak/context";

/**
 * Middleware to serve static files from specified directories and fall back to an `index.html` file for Single Page Applications (SPAs).
 *
 * This middleware serves requests for static assets (e.g., JavaScript, CSS, images) from one or more provided static paths.
 * If no static asset matches a given request path, the middleware falls back to serving `index.html` from the first path in `staticPaths`.
 * This setup is commonly used in SPAs that rely on client-side routing, such as those built with React or Vue.
 *
 * ### Parameters:
 * - `staticPaths` (string[]): An array of paths (directories) where static assets (e.g., `dist/`, `static/`) are stored.
 *   The first path in the array is used to serve `index.html` as the fallback file.
 *
 * ### Usage:
 * ```typescript
 * import routeStaticFilesFrom from './path/to/this/middleware';
 * 
 * app.use(routeStaticFilesFrom(['dist', 'static']));
 * ```
 *
 * ### Important Security Considerations:
 * - **Directory Traversal**: Ensure that the `context.send()` function from your framework properly sanitizes paths and prevents directory traversal attacks. Malicious actors could otherwise try to access files outside the allowed static paths using `../` sequences.
 * - **Sensitive Files Exposure**: Avoid placing sensitive files (e.g., `.env`, `config.yaml`) within or near `staticPaths` directories, as they could be served if accidentally requested by clients.
 * - **Allowed File Extensions**: The middleware only serves files with specific extensions (e.g., `.js`, `.css`, `.png`, `.jpg`) to reduce the risk of unintended file exposure. Avoid allowing access to other types of files, such as `.json` or `.html`, which could reveal internal data or configuration.
 * - **Error Handling and Logging**: Avoid logging detailed error messages, paths, or stack traces in production environments to prevent leakage of sensitive information.
 * - **Rate Limiting**: Consider adding rate limiting middleware to prevent potential Denial-of-Service (DoS) attacks where large or repeated requests could exhaust server resources.
 * - **Cache Control**: Set appropriate `Cache-Control` headers for static assets. For example, long cache durations (`max-age=31536000`) are recommended for assets like `.js` and `.css` files, while short or no caching (`no-store, no-cache`) is preferred for `index.html` to ensure users receive the latest content.
 * - **Path Validation**: Avoid symlink traversal issues by validating that files being served reside strictly within the allowed static paths. If necessary, verify that any file accessed is within the intended directory structure and not accessible via a symbolic link.
 *
 *
 * ### Notes:
 * This middleware is optimized for Single Page Applications and is intended for environments where static assets are isolated in
 * specific directories. Carefully review the security recommendations and test your application’s behavior to avoid unintended file exposures.
 */
export default function routeStaticFilesFrom(staticPaths: string[]) {
  return async (context: Context<Record<string, object>>, next: Next) => {
    const requestUrl = context.request.url.pathname;
    console.log("Request URL:", requestUrl);

    // Allow only common asset extensions to mitigate serving unintended files
    const allowedExtensions = /\.(js|css|png|jpg|svg|jpeg|gif|ico)$/i;

    // Only process routes under /assets/ and ignore any root or non-asset routes
    if (allowedExtensions.test(requestUrl)) {
      console.log("Serving static file:", requestUrl);
      for (const path of staticPaths) {
        try {
          await context.send({ root: path });
          console.log(`File found and served from: ${requestUrl}`);
          return; // Serve the file if found and stop further processing
        } catch {
          continue; // If file not found, try the next path
        }
      }
    }

    // If no static file is found, serve index.html for frontend routes
    console.log("No specific file found; serving index.html as fallback.");
    if (!requestUrl.includes(".")){
        try {
          await context.send({
            root: staticPaths[0], // Only use the first path as the root for index.html
            path: "index.html",    // Force index.html as the file served
          });
          console.log("Index.html successfully sent as fallback");
        } catch (err) {
          console.error("Error serving index.html:", err);
          context.response.status = 404;
          context.response.body = "404 - Not Found";
        }
    } else {
      await next();
    }
  };
}

