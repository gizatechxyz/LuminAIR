"use client";

import { VerifyButton, VerifyBadge } from "@gizatech/luminair-react";
import "@gizatech/luminair-react/styles.css";
import { useState, useEffect } from "react";
import { Sun, Moon } from "lucide-react";

export default function Home() {
  const [isDarkMode, setIsDarkMode] = useState(false);

  // Initialize theme from localStorage or system preference
  useEffect(() => {
    const savedTheme = localStorage.getItem("theme");
    const prefersDark = window.matchMedia(
      "(prefers-color-scheme: dark)"
    ).matches;

    if (savedTheme === "dark" || (!savedTheme && prefersDark)) {
      setIsDarkMode(true);
      document.documentElement.classList.add("dark");
    } else {
      setIsDarkMode(false);
      document.documentElement.classList.remove("dark");
    }
  }, []);

  const toggleTheme = () => {
    const newIsDarkMode = !isDarkMode;
    setIsDarkMode(newIsDarkMode);

    if (newIsDarkMode) {
      document.documentElement.classList.add("dark");
      localStorage.setItem("theme", "dark");
    } else {
      document.documentElement.classList.remove("dark");
      localStorage.setItem("theme", "light");
    }
  };

  return (
    <div className="min-h-screen bg-white dark:bg-black flex items-center justify-center p-8 transition-colors relative">
      {/* Theme Toggle Button */}
      <button
        onClick={toggleTheme}
        className="absolute top-8 right-8 p-2 rounded-lg bg-gray-100 dark:bg-gray-900 hover:bg-gray-200 dark:hover:bg-gray-800 transition-colors"
        aria-label="Toggle theme"
      >
        {isDarkMode ? (
          <Sun className="h-5 w-5 text-yellow-500" />
        ) : (
          <Moon className="h-5 w-5 text-gray-600" />
        )}
      </button>

      <div className="max-w-4xl w-full text-center space-y-12">
        <div className="pt-4 flex justify-center">
          <VerifyButton
            graphPath="/graph.dot"
            proofPath="/proof"
            settingsPath="/settings"
          />
        </div>

        <div className="pt-6 flex justify-center space-x-4">
          <VerifyBadge
            proofPath="/proof"
            settingsPath="/settings"
            graphPath="/graph.dot"
            labelText="VERIFIED COMPUTE"
          />
        </div>
        <div className="pt-8 text-xs text-gray-400 dark:text-gray-500 font-mono">
          Built with LuminAIR & STWO
        </div>
      </div>
    </div>
  );
}
