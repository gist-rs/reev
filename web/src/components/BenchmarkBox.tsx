// BenchmarkBox component for individual 16x16 result display

import { BenchmarkResult } from "../types/benchmark";

interface BenchmarkBoxProps {
  result: BenchmarkResult;
  size?: number;
  onClick?: (result: BenchmarkResult) => void;
  className?: string;
  isRunning?: boolean;
  isSelected?: boolean;
}

export function BenchmarkBox({
  result,
  size = 16,
  onClick,
  className = "",
  isRunning = false,
  isSelected = false,
}: BenchmarkBoxProps) {
  const getColorClass = (result: BenchmarkResult): string => {
    // If running, don't apply static background color - animation will handle it
    if (isRunning) return "";

    // Use color_class if specified, otherwise fall back to score-based logic
    if (result.color_class === "gray") return "bg-gray-400";
    if (result.color_class === "green") return "bg-green-500";
    if (result.color_class === "yellow") return "bg-yellow-500";
    if (result.color_class === "red") return "bg-red-500";

    // Fallback to score-based logic
    if (result.score >= 1.0) return "bg-green-500"; // 100%
    if (result.score >= 0.25) return "bg-yellow-500"; // <100% but >=25%
    return "bg-red-500"; // <25%
  };

  const getAnimationClass = () => {
    if (isRunning) {
      return "animate-blink-fade";
    }
    return "";
  };

  const baseClasses = `${getColorClass(result)} hover:opacity-80 transition-opacity cursor-pointer ${isSelected ? "ring-2 ring-blue-500 ring-offset-1" : ""}`;
  const styleProps = {
    width: `${size}px`,
    height: `${size}px`,
    margin: "1px", // 2px gap achieved with 1px margin
    borderRadius: "2px",
  };

  const animationClass = getAnimationClass();

  return (
    <div
      className={`${baseClasses} ${className} ${animationClass} relative group active:scale-95 transition-transform ring-2 ring-transparent hover:ring-gray-400 active:ring-gray-600`}
      style={{
        ...styleProps,
        minWidth: "16px",
        minHeight: "16px",
      }}
      onClick={() => onClick && onClick(result)}
    />
  );
}
