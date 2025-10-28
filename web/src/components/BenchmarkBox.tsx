// BenchmarkBox component for individual 16x16 result display

import { BenchmarkResult } from "../types/benchmark";
import { getBenchmarkColorClass } from "../utils/benchmarkColors";

interface BenchmarkBoxProps {
  result: BenchmarkResult;
  size?: number;
  onClick?: (result: BenchmarkResult) => void;
  className?: string;
  isRunning?: boolean;
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
    // HARDCODED TEST: Keep 001 hardcoded, add 002 for dynamic testing
    if (
      result.agent_type === "deterministic" &&
      !result.isEmpty && // Only first row is not a placeholder
      (result.benchmark_id === "001-sol-transfer" || // Keep 001 hardcoded
        (result.benchmark_id.includes("002") && isRunning)) // Dynamic test for 002 - only if running
    ) {
      console.log(
        `🧪 [BenchmarkBox] HARDCODED TEST: Applying animation to ${result.benchmark_id}, agent ${result.agent_type}, date ${result.timestamp?.substring(0, 10)}`,
      );
      return "animate-blink-fade";
    }

    if (isRunning) {
      return "animate-blink-fade";
    }
    return "";
  };

  // HARDCODED TEST: Override background color for deterministic 001-sol-transfer
  const shouldOverrideColor =
    result.agent_type === "deterministic" &&
    result.benchmark_id === "001-sol-transfer";

  const baseClasses = `${shouldOverrideColor ? "" : getBenchmarkColorClass(result, isRunning)} ${disabled ? "cursor-not-allowed opacity-50" : "hover:opacity-80 cursor-pointer"} transition-opacity ${isSelected ? "ring-2 ring-blue-500 ring-offset-1" : ""}`;
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
