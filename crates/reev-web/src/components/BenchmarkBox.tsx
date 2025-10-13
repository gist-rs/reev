// BenchmarkBox component for individual 16x16 result display

import { BenchmarkResult } from "../types/benchmark";

interface BenchmarkBoxProps {
  result: BenchmarkResult;
  size?: number;
  onClick?: (result: BenchmarkResult) => void;
  className?: string;
}

export function BenchmarkBox({
  result,
  size = 16,
  onClick,
  className = "",
}: BenchmarkBoxProps) {
  const getColorClass = (score: number): string => {
    if (score >= 1.0) return "bg-green-500"; // 100%
    if (score >= 0.25) return "bg-yellow-500"; // <100% but >=25%
    return "bg-red-500"; // <25%
  };

  const handleClick = () => {
    if (onClick) {
      onClick(result);
    }
  };

  const baseClasses = `${getColorClass(result.score)} hover:opacity-80 transition-opacity cursor-pointer`;
  const styleProps = {
    width: `${size}px`,
    height: `${size}px`,
    margin: "1px", // 2px gap achieved with 1px margin
    borderRadius: "2px",
  };

  return (
    <div
      className={`${baseClasses} ${className}`}
      style={styleProps}
      onClick={handleClick}
      title={`${result.benchmark_id}: ${(result.score * 100).toFixed(1)}% - ${result.agent_type}`}
    ></div>
  );
}
