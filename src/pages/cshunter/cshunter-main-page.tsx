import { Icon } from "@iconify/react/dist/iconify.js";
import Sidebar from "../../components/sidebar/sidebar";
import SidebarProvider from "../../components/sidebar/sidebar-provider";
import { Page } from "../../utils/types";

const pages: Page[] = [
    { 
        name: "Главная", 
        icon: <Icon width="30px" height="30px" icon="material-symbols:home-rounded"></Icon>, 
        source: <div>test</div>
    }
];

export default function CSHunterMainPage() {
    return (
        <Sidebar pages={pages} provider={SidebarProvider} />
    )
}