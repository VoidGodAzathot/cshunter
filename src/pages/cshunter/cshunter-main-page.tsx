import { Icon } from "@iconify/react/dist/iconify.js";
import Sidebar from "../../components/sidebar/sidebar";
import SidebarProvider from "../../components/sidebar/sidebar-provider";
import { Page } from "../../utils/types";
import CSHunterBrowsersPage from "./cshunter-browsers-page";
import CSHunterAnalyzerPage from "./cshunter-analyzer-page";

const pages: Page[] = [
    { 
        name: "Главная", 
        icon: <Icon width="30px" height="30px" icon="material-symbols:home-rounded"></Icon>, 
        source: () => <div>test</div>
    },
    { 
        name: "Браузеры", 
        icon: <Icon width="30px" height="30px" icon="stash:browser-solid"></Icon>, 
        source: CSHunterBrowsersPage
    },
    { 
        name: "Анализ", 
        icon: <Icon width="30px" height="30px" icon="ix:analyze"></Icon>, 
        source: CSHunterAnalyzerPage
    }
];

export default function CSHunterMainPage() {
    return (
        <Sidebar pages={pages} provider={SidebarProvider} />
    )
}