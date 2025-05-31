import { Calendar, ChevronDown, Home, Inbox, Search, Settings } from "lucide-react"

import {
  Sidebar,
  SidebarContent,
  SidebarGroup,
  SidebarGroupContent,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarTrigger,
} from "@/components/ui/sidebar"
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from "./ui/dropdown-menu"
import { useNavigate, useParams } from "react-router-dom"
import { useEffect, useState } from "react"

// Menu items.
const items = [
  {
    title: "Overview",
    url: "#",
    icon: Home,
  },
  {
    title: "Resources",
    url: "#",
    icon: Inbox,
  },
  {
    title: "Calendar",
    url: "#",
    icon: Calendar,
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
// TODO: The assets should be fetched not hardcoded.
// TODO: The assets available should reflect only what the user has access to.
export function AppSidebar() {
  const navigate = useNavigate();
  const [workspace, setWorkspace] = useState<string | null>(null);
  const { asset } = useParams<{ asset: string}>();

  useEffect(() => {
    if (asset) {
      setWorkspace(asset);
    } else {
      setWorkspace("Select Workspace");
    }
    
  }, [asset])

  const handleSelectAsset = (ws: string) => {
    navigate(`/dashboard/${ws}`)
    setWorkspace(ws);
  }
  return (
    <Sidebar collapsible="icon">
      <SidebarContent>
        <SidebarMenu>
          <SidebarMenuItem>
            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton>
                  <span className="font-bold px-2">{workspace}</span>
                  <ChevronDown className="ml-auto" />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent className="w[--radix-popper-anchor-width]">
                <DropdownMenuItem onClick={() => handleSelectAsset("DF")}>
                  <span>DF</span>
                </DropdownMenuItem>
                <DropdownMenuItem onClick={() => handleSelectAsset("TL")}>
                  <span>TL</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </SidebarMenuItem>
        </SidebarMenu>
                
        <SidebarGroup>
          <SidebarGroupContent>
            <SidebarMenu>
              {items.map((item) => (
                <SidebarMenuItem key={item.title}>
                  <SidebarMenuButton asChild>
                    <a href={item.url}>
                      <item.icon />
                      <span>{item.title}</span>
                    </a>
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

