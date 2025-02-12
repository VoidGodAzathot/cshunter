export enum Flag {
  SYSTEM,
  NTFS,
}

export type Volume = {
  path: string;
  free_space: number;
  available_space: number;
  total_space: number;
  flags: Flag[];
};

export type FileRecord = {
  name: string;
  path: string | undefined;
  timestamp: number;
  reason: string;
};

export type SteamAccount = {
  id: number;
  persona_name: string;
  account_name: string;
  timestamp: string;
  most_recent: string;
};

export enum Driver {
  BLINK,
}

export type Browser = {
  id: string;
  path: string;
  driver: Driver;
  support: boolean;
};

export type DownloadDat = {
  browser: string;
  file: string;
  url: string;
  imestamp: number;
};

export type VisitDat = {
  browser: string;
  title: string;
  url: string;
  timestamp: number;
};

export type CacheDat = {
  browser: string;
  url: string;
};

export type AnalyzeContext = {
  items: ItemContext[];
};

export type ItemContext = {
  name: string;
  path: string;
  size: number;
  crc32: number;
};

export type Page = {
  source: () => JSX.Element;
  icon: JSX.Element;
  name: string;
};

export type StorageUpdate = {
  name: string;
};

export type MiniDat = {
  value: string;
  id: string;
};

export type MiniDatInfo = {
  id: string;
  name: string;
  description: string;
  filtering: boolean;
  stable: boolean;
};

export type ShellBagView = {
  path: string;
  name: string;
  timestamp: number;
  action: ShellBagViewAction;
};

export enum ShellBagViewAction {
  DELETE,
  MODIFY,
  ACCESS,
  CREATE,
}

export type ShellBagDat = {
  value: string;
  num: number;
  timestamp: number;
};

export type Strings = {
  address: string;
  values: string[];
};

export type ModuleStrings = {
  address: string;
  values: string[];
  module: string;
};
