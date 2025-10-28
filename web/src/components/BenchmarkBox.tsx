// BenchmarkBox component for individual 16x16 result display

import { BenchmarkResult } from "../types/benchmark";
import { getBenchmarkColorClass } from "../utils/benchmarkColors";

interface BenchmarkBoxProps {
  result: BenchmarkResult;
  size?: number;
  onClick?: (result: BenchmarkResult) => void;
  className?: string;
  isRunning?: boolean;
  isAnyRunning?: boolean;
  isSelected?: boolean;
  disabled?: boolean;
  showDate?: boolean;
}

export function BenchmarkBox({
  result,
  size = 16,
  onClick,
  className = "",
  isRunning = false,
  isSelected = false,
  disabled = false,
  showDate = false,
}: BenchmarkBoxProps) {
  const getAnimationClass = () => {
    if (isRunning) {
      return "animate-blink-fade";
    }
    return "";
  };

  const baseClasses = `${getBenchmarkColorClass(result, isRunning)} ${disabled ? "cursor-not-allowed opacity-50" : "hover:opacity-80 cursor-pointer"} transition-opacity ${isSelected ? "ring-2 ring-blue-500 ring-offset-1" : ""}`;
  const styleProps = {
    width: `${size}px`,
    height: `${size}px`,
    margin: "1px", // 2px gap achieved with 1px margin
    borderRadius: "2px",
  };

  const animationClass = getAnimationClass();

  const dateText =
    showDate && result.timestamp
      ? result.timestamp.substring(8, 10) // Just the day number
      : "";

  const finalClassName = `${baseClasses} ${className} ${animationClass} relative group ${disabled ? "" : "active:scale-95 transition-transform ring-2 ring-transparent hover:ring-gray-400 active:ring-gray-600"} flex items-center justify-center overflow-hidden`;

  return (
    <div
      className={finalClassName}
      style={{
        ...styleProps,
        minWidth: "16px",
        minHeight: "16px",
      }}
      onClick={() => !disabled && onClick && onClick(result)}
    >
      {showDate && dateText && (
        <span
          className="text-white font-mono leading-none"
          style={{
            fontSize: "7px",
            lineHeight: "1",
            whiteSpace: "nowrap",
            maxWidth: "14px",
            fontWeight: "bold",
          }}
        ></span>
      )}
    </div>
  );
}
