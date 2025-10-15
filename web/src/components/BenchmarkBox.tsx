// BenchmarkBox component for individual 16x16 result display

import { BenchmarkResult } from "../types/benchmark";

interface BenchmarkBoxProps {
  result: BenchmarkResult;
  size?: number;
  onClick?: (result: BenchmarkResult) => void;
  className?: string;
  isRunning?: boolean;
}

export function BenchmarkBox({
  result,
  size = 16,
  onClick,
  className = "",
  isRunning = false,
}: BenchmarkBoxProps) {
  const getColorClass = (result: BenchmarkResult): string => {
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
      return "animate-pulse bg-gradient-to-r from-[#9945FF] to-[#00D18C] bg-size-200 bg-pos-0";
    }
    return "";
  };

  // No need for API calls - all info is in memory

  const handleClick = () => {
    if (onClick) {
      onClick(result);
    }
  };

  const baseClasses = `${getColorClass(result)} hover:opacity-80 transition-opacity cursor-pointer`;
  const styleProps = {
    width: `${size}px`,
    height: `${size}px`,
    margin: "1px", // 2px gap achieved with 1px margin
    borderRadius: "2px",
  };

  return (
    <div
      className={`${baseClasses} ${className} ${getAnimationClass()} relative group active:scale-95 transition-transform`}
      style={{
        ...styleProps,
        minWidth: "20px", // Larger touch target
        minHeight: "20px", // Larger touch target
        ...(isRunning && {
          background:
            "linear-gradient(90deg, #9945FF 0%, #00D18C 50%, #9945FF 100%)",
          backgroundSize: "200% 100%",
          animation: "shimmer 2s ease-in-out infinite",
        }),
      }}
      onClick={() => onClick && onClick(result)}
    >
      <div className="absolute -inset-1 rounded-sm border-2 border-transparent group-hover:border-gray-400 group-active:border-gray-600 transition-colors duration-150"></div>
    </div>
  );
}
