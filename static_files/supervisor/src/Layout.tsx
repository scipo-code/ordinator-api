import React from "react";
import { SidebarProvider  } from "./components/ui/sidebar";
import { AppSidebar } from "@/components/app-sidebar"
import { Outlet } from "react-router-dom";


export default function Layout() {
  return (
    <SidebarProvider>
      <AppSidebar />
        <main className="flex-1">
          <Outlet />
        </main>
    </SidebarProvider>
  )
}
