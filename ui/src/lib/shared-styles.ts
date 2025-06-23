import { cn } from "./utils";

export const getSharedButtonStyles = (className?: string) => {
  return cn(
    "relative bg-black text-white hover:bg-gray-800 dark:bg-white dark:text-black dark:hover:bg-gray-200 transition-colors font-mono text-sm px-6 py-3 rounded-sm border border-gray-200 dark:border-gray-700",
    className
  );
};

export const baseButtonStyles = "relative bg-black text-white hover:bg-gray-800 dark:bg-white dark:text-black dark:hover:bg-gray-200 transition-colors font-mono text-sm px-6 py-3 rounded-sm border border-gray-200 dark:border-gray-700"; 