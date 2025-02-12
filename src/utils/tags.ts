export type Tag = {
  msg: string;
  id: string;
  desc: string;
  color: "red" | "green" | "blue";
};

export const Tags: Tag[] = [
  {
    msg: "vm env",
    desc: "Программа запущена в виртуальной среде.",
    id: "vmd_verdict",
    color: "red",
  },
  {
    msg: "bad version",
    desc: "Вы используете неверную версию программы.",
    id: "no_last_version",
    color: "blue",
  }
];

export function tryApplyTag(arr: Tag[], id: string): Tag[] {
  if (arr.filter((tag) => tag.id === id).length == 0) {
    const tag = Tags.filter((tag) => tag.id === id);
    if (tag.length == 0) return arr;
    return [...arr, ...[tag[0]]];
  }
  return arr;
}
