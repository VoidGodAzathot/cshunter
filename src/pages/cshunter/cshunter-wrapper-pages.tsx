import { Icon } from "@iconify/react/dist/iconify.js";
import Sidebar from "../../components/sidebar/sidebar";
import SidebarProvider from "../../components/sidebar/sidebar-provider";
import { Page } from "../../utils/types";
import CSHunterBrowsersPage from "./cshunter-browsers-page";
import CSHunterAnalyzerPage from "./cshunter-analyzer-page";
import CSHunterDelFilesPage from "./cshunter-delfiles-page";
import CSHunterSteamAccPage from "./cshunter-steamacc-page";
import CSHunterMiniDatPage from "./cshunter-mini-dat-page";
import CSHunterDriversPage from "./cshunter-drivers-page";

const pages: Page[] = [
  {
    name: "Данные об использовании",
    icon: (
      <Icon width="30px" height="30px" icon="fluent:data-pie-20-filled"></Icon>
    ),
    source: CSHunterMiniDatPage,
  },
  {
    name: "Стим Аккаунты",
    icon: <Icon width="30px" height="30px" icon="ri:steam-fill"></Icon>,
    source: CSHunterSteamAccPage,
  },
  {
    name: "Браузеры",
    icon: <Icon width="30px" height="30px" icon="stash:browser-solid"></Icon>,
    source: CSHunterBrowsersPage,
  },
  {
    name: "Анализ",
    icon: <Icon width="30px" height="30px" icon="ix:analyze"></Icon>,
    source: CSHunterAnalyzerPage,
  },
  {
    name: "Удаленные файлы",
    icon: (
      <Icon width="30px" height="30px" icon="fluent:delete-16-filled"></Icon>
    ),
    source: CSHunterDelFilesPage,
  },
  {
    name: "Драйверы",
    icon: (
      <Icon width="30px" height="30px" icon="material-symbols:license-rounded"></Icon>
    ),
    source: CSHunterDriversPage,
  },
];

export default function CSHunterWrapperPages() {
  return <Sidebar pages={pages} provider={SidebarProvider} />;
}
