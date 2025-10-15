// BenchmarkBox component for individual 16x16 result display

import { useState } from "react";
import { BenchmarkResult, BenchmarkDetails } from "../types/benchmark";
import { Tooltip } from "./ui/Tooltip";
import { useBenchmarkDetails } from "../hooks/useBenchmarkDetails";

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
  const { fetchBenchmarkDetails, getCachedDetails } = useBenchmarkDetails();
  const [isLoading, setIsLoading] = useState(false);
  const benchmarkDetails = getCachedDetails(result.benchmark_id);

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

  const handleMouseEnter = () => {
    if (!benchmarkDetails && !isLoading) {
      setIsLoading(true);
      fetchBenchmarkDetails(result.benchmark_id)
        .then((details) => {
          if (details) {
            console.log(`Loaded details for ${result.benchmark_id}:`, details);
          }
        })
        .catch((error) => {
          console.error(
            `Failed to load details for ${result.benchmark_id}:`,
            error,
          );
        })
        .finally(() => {
          setIsLoading(false);
        });
    }
  };

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

  const tooltipContent = (
    <div className="text-center">
      <div className="font-semibold text-white mb-1">{result.benchmark_id}</div>
      {isLoading ? (
        <div className="text-gray-300 text-xs">Loading...</div>
      ) : benchmarkDetails ? (
        <>
          <div className="text-gray-300 text-xs mb-2 max-w-xs">
            {benchmarkDetails.description === "Failed to load description" ? (
              <div>
                <div className="text-red-400">API Error</div>
                <div className="text-gray-400">
                  Check benchmark YAML structure
                </div>
              </div>
            ) : (
              benchmarkDetails.description
            )}
          </div>
          {benchmarkDetails.tags.length > 0 && (
            <div className="flex flex-wrap gap-1 justify-center mb-2">
              {benchmarkDetails.tags.slice(0, 3).map((tag, index) => (
                <span
                  key={index}
                  className="bg-blue-600 text-white text-xs px-2 py-1 rounded"
                >
                  {tag}
                </span>
              ))}
              {benchmarkDetails.tags.length > 3 && (
                <span className="text-gray-400 text-xs">
                  +{benchmarkDetails.tags.length - 3} more
                </span>
              )}
            </div>
          )}
          <div className="text-white font-medium">
            Score: {(result.score * 100).toFixed(1)}%
          </div>
          <div className="text-gray-300 text-xs">
            Agent: {result.agent_type}
          </div>
          <div className="text-gray-400 text-xs">
            Status: {result.final_status}
          </div>
        </>
      ) : (
        <div className="text-gray-300 text-xs">Hover to load details...</div>
      )}
    </div>
  );

  return (
    <Tooltip content={tooltipContent} delay={300} position="top">
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
        onClick={handleClick}
        onMouseEnter={handleMouseEnter}
      >
        {/* Mobile touch indicator - subtle pulse effect */}
        <div className="absolute inset-0 rounded-sm opacity-0 group-hover:opacity-20 group-active:opacity-30 bg-white transition-opacity duration-200"></div>
        {/* Touch feedback ring */}
        <div className="absolute -inset-1 rounded-sm border-2 border-transparent group-hover:border-gray-400 group-active:border-gray-600 transition-colors duration-150"></div>
      </div>
    </Tooltip>
  );
}
