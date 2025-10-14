import React, { useState, useRef, useEffect } from "react";

interface TooltipProps {
  content: React.ReactNode;
  children: React.ReactNode;
  className?: string;
  delay?: number;
  position?: "top" | "bottom" | "left" | "right";
  mobilePosition?: "center" | "top" | "bottom";
}

export function Tooltip({
  content,
  children,
  className = "",
  delay = 300,
  position = "top",
  mobilePosition = "center",
}: TooltipProps) {
  const [isVisible, setIsVisible] = useState(false);
  const [coords, setCoords] = useState({ x: 0, y: 0 });
  const timeoutRef = useRef<number>();
  const containerRef = useRef<HTMLDivElement>(null);
  const touchTimeoutRef = useRef<number>();

  const calculatePosition = (rect: DOMRect) => {
    let x = 0;
    let y = 0;

    switch (position) {
      case "top":
        x = rect.left + rect.width / 2;
        y = rect.top;
        break;
      case "bottom":
        x = rect.left + rect.width / 2;
        y = rect.bottom;
        break;
      case "left":
        x = rect.left;
        y = rect.top + rect.height / 2;
        break;
      case "right":
        x = rect.right;
        y = rect.top + rect.height / 2;
        break;
    }

    return { x, y };
  };

  const calculateMobilePosition = (
    touchX: number,
    touchY: number,
    rect: DOMRect,
  ) => {
    switch (mobilePosition) {
      case "center":
        // Center tooltip on touch point, but ensure it stays on screen
        const tooltipWidth = 320; // max-w-xs = 320px
        const tooltipHeight = 120; // approximate height
        const x = Math.max(
          tooltipWidth / 2,
          Math.min(window.innerWidth - tooltipWidth / 2, touchX),
        );
        const y = Math.max(
          tooltipHeight / 2 + 50,
          Math.min(window.innerHeight - tooltipHeight / 2 - 50, touchY),
        );
        return { x, y };
      case "top":
        return { x: touchX, y: touchY - 60 };
      case "bottom":
        return { x: touchX, y: touchY + 60 };
      default:
        return { x: touchX, y: touchY - 60 };
    }
  };

  const showTooltip = (clientX?: number, clientY?: number) => {
    const rect = containerRef.current?.getBoundingClientRect();
    if (!rect) return;

    let coords;
    if (clientX !== undefined && clientY !== undefined) {
      // For touch events, use the touch coordinates with mobile positioning
      coords = calculateMobilePosition(clientX, clientY, rect);
    } else {
      // For mouse events, calculate based on element position
      coords = calculatePosition(rect);
    }

    setCoords(coords);

    timeoutRef.current = setTimeout(() => {
      setIsVisible(true);
    }, delay);
  };

  const hideTooltip = () => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setIsVisible(false);
  };

  const handleMouseEnter = (e: React.MouseEvent) => {
    showTooltip();
  };

  const handleMouseLeave = () => {
    hideTooltip();
  };

  const handleTouchStart = (e: React.TouchEvent) => {
    e.preventDefault(); // Prevent mouse events from firing
    const touch = e.touches[0];
    showTooltip(touch.clientX, touch.clientY);
  };

  const handleTouchEnd = (e: React.TouchEvent) => {
    e.preventDefault();
    // Auto-hide after 3 seconds on touch
    touchTimeoutRef.current = setTimeout(() => {
      hideTooltip();
    }, 3000);
  };

  useEffect(() => {
    // Add global touch listener to dismiss tooltips on tap outside
    const handleGlobalTouch = (e: any) => {
      if (isVisible && containerRef.current) {
        const touch = e.touches[0];
        const element = document.elementFromPoint(touch.clientX, touch.clientY);
        if (!containerRef.current.contains(element as Node)) {
          hideTooltip();
        }
      }
    };

    const handleGlobalClick = (e: any) => {
      if (isVisible && containerRef.current) {
        const element = e.target as Node;
        if (!containerRef.current.contains(element)) {
          hideTooltip();
        }
      }
    };

    document.addEventListener("touchstart", handleGlobalTouch, {
      passive: true,
    });
    document.addEventListener("click", handleGlobalClick);

    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      if (touchTimeoutRef.current) {
        clearTimeout(touchTimeoutRef.current);
      }
      document.removeEventListener("touchstart", handleGlobalTouch);
      document.removeEventListener("click", handleGlobalClick);
    };
  }, [isVisible]);

  const getPositionClasses = () => {
    // Check if this is a touch-triggered tooltip (no rect-based positioning)
    const isTouchTooltip =
      coords.x > 0 &&
      coords.y > 0 &&
      Math.abs(
        coords.x - (containerRef.current?.getBoundingClientRect().left || 0),
      ) > 100;

    if (isTouchTooltip) {
      // For touch events, center the tooltip on the touch point
      return "transform -translate-x-1/2 -translate-y-1/2";
    }

    switch (position) {
      case "top":
        return "transform -translate-x-1/2 -translate-y-full mb-2";
      case "bottom":
        return "transform -translate-x-1/2 mt-2";
      case "left":
        return "transform -translate-y-1/2 -translate-x-full mr-2";
      case "right":
        return "transform -translate-y-1/2 ml-2";
      default:
        return "transform -translate-x-1/2 -translate-y-full mb-2";
    }
  };

  const getArrowClasses = () => {
    // Check if this is a touch-triggered tooltip
    const isTouchTooltip =
      coords.x > 0 &&
      coords.y > 0 &&
      Math.abs(
        coords.x - (containerRef.current?.getBoundingClientRect().left || 0),
      ) > 100;

    if (isTouchTooltip) {
      // Hide arrow for touch-centered tooltips
      return "hidden";
    }

    switch (position) {
      case "top":
        return "top-full left-1/2 transform -translate-x-1/2 border-l-transparent border-r-transparent border-b-transparent border-t-gray-900";
      case "bottom":
        return "bottom-full left-1/2 transform -translate-x-1/2 border-l-transparent border-r-transparent border-t-transparent border-b-gray-900";
      case "left":
        return "left-full top-1/2 transform -translate-y-1/2 border-t-transparent border-b-transparent border-r-transparent border-l-gray-900";
      case "right":
        return "right-full top-1/2 transform -translate-y-1/2 border-t-transparent border-b-transparent border-l-transparent border-r-gray-900";
      default:
        return "top-full left-1/2 transform -translate-x-1/2 border-l-transparent border-r-transparent border-b-transparent border-t-gray-900";
    }
  };

  return (
    <>
      <div
        ref={containerRef}
        className={`inline-block ${className} touch-manipulation`}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        onTouchStart={handleTouchStart}
        onTouchEnd={handleTouchEnd}
        style={{ touchAction: "manipulation" }}
      >
        {children}
      </div>

      {isVisible && (
        <div
          className="fixed z-50"
          style={{
            left: coords.x,
            top: coords.y,
            pointerEvents: "auto",
          }}
        >
          <div className={`relative ${getPositionClasses()}`}>
            <div className="bg-gray-900 text-white text-sm rounded-lg px-3 py-2 max-w-xs shadow-lg backdrop-blur-sm">
              {content}
            </div>
            <div
              className={`absolute w-0 h-0 border-4 ${getArrowClasses()}`}
            ></div>
          </div>
        </div>
      )}
    </>
  );
}
