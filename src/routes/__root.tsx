import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";

export const Route = createRootRoute({
  component: () => (
    <>
      <main className="h-dvh max-h-screen w-full max-w-screen overflow-auto p-2">
        <Outlet />
      </main>

      <TanStackRouterDevtools />
    </>
  ),
});
