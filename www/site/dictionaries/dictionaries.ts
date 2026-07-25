import "server-only";

const dictionaries = {
  en: () => import("./en.json").then((module) => module.default),
  tr: () => import("./tr.json").then((module) => module.default),
};

export type Dictionary = Awaited<ReturnType<(typeof dictionaries)["tr"]>>;

export const getDictionary = async (locale: "en" | "tr") =>
  dictionaries[locale]?.() ?? dictionaries.en();
