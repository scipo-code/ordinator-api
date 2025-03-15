import { Calendar, Search, Settings, User } from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar"
import { Link, useParams } from "react-router-dom"

// Menu items.
// TODO: The assets available should reflect only what the user has access to.
export function AppSidebar() {
  const { asset } = useParams();
  const items = [
    {
      title: "Planning",
      url: `/${asset}/planning`,
      icon: Calendar,
    },
    {
      title: "Resources",
      url: `/${asset}/resources`,
      icon: User,
    },
    {
      title: "Search",
      url: "#",
      icon: Search,
    },
    {
      title: "Settings",
      url: "#",
      icon: Settings,
    },
  ]
  return (
    <Sidebar collapsible="none" className="min-h-screen">
      <SidebarContent>
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <Link to={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </Link>
                  </SidebarMenuButton>
                </SidebarMenuItem>
              ))}
            </SidebarMenu>
          </SidebarGroupContent>
        </SidebarGroup>
      </SidebarContent>
    </Sidebar>
  )
}

