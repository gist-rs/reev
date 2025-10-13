// Custom hook for responsive layout detection and management

import { useState, useEffect } from 'preact/hooks';

export type Breakpoint = 'mobile' | 'tablet' | 'desktop';

interface ResponsiveLayout {
  breakpoint: Breakpoint;
  isMobile: boolean;
  isTablet: boolean;
  isDesktop: boolean;
  width: number;
  height: number;
}

const MOBILE_BREAKPOINT = 768;
const TABLET_BREAKPOINT = 1024;

export function useResponsiveLayout(): ResponsiveLayout {
  const [layout, setLayout] = useState<ResponsiveLayout>(() => {
    if (typeof window === 'undefined') {
      return {
        breakpoint: 'desktop',
        isMobile: false,
        isTablet: false,
        isDesktop: true,
        width: 1024,
        height: 768,
      };
    }

    const width = window.innerWidth;
    const height = window.innerHeight;
    const breakpoint = getBreakpoint(width);

    return {
      breakpoint,
      isMobile: breakpoint === 'mobile',
      isTablet: breakpoint === 'tablet',
      isDesktop: breakpoint === 'desktop',
      width,
      height,
    };
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const handleResize = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      const breakpoint = getBreakpoint(width);

      setLayout({
        breakpoint,
        isMobile: breakpoint === 'mobile',
        isTablet: breakpoint === 'tablet',
        isDesktop: breakpoint === 'desktop',
        width,
        height,
      });
    };

    // Add event listener
    window.addEventListener('resize', handleResize);

    // Initial call to set correct state
    handleResize();

    // Cleanup
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return layout;
}

function getBreakpoint(width: number): Breakpoint {
  if (width < MOBILE_BREAKPOINT) {
    return 'mobile';
  } else if (width < TABLET_BREAKPOINT) {
    return 'tablet';
  } else {
    return 'desktop';
  }
}

// Utility function to get responsive values
export function getResponsiveValue<T>(
  values: Partial<Record<Breakpoint, T>>,
  breakpoint: Breakpoint,
  defaultValue: T
): T {
  return values[breakpoint] ??
         values.desktop ??
         values.tablet ??
         values.mobile ??
         defaultValue;
}

// Hook for responsive grid layout
export function useResponsiveGridLayout() {
  const { breakpoint, isMobile, isDesktop } = useResponsiveLayout();

  const getGridConfig = () => {
    if (isMobile) {
      return {
        columns: 1,
        gap: 2,
        boxSize: 16,
        containerClass: 'flex flex-col gap-6',
      };
    } else {
      return {
        columns: 4,
        gap: 1,
        boxSize: 16,
        containerClass: 'flex flex-col gap-4',
      };
    }
  };

  return {
    ...getGridConfig(),
    breakpoint,
    isMobile,
    isDesktop,
  };
}

// Hook for responsive font sizes
export function useResponsiveTypography() {
  const { breakpoint } = useResponsiveLayout();

  const getFontSizes = () => {
    switch (breakpoint) {
      case 'mobile':
        return {
          heading: 'text-lg font-bold',
          subheading: 'text-base font-semibold',
          body: 'text-sm',
          caption: 'text-xs',
        };
      case 'tablet':
        return {
          heading: 'text-xl font-bold',
          subheading: 'text-lg font-semibold',
          body: 'text-base',
          caption: 'text-sm',
        };
      case 'desktop':
      default:
        return {
          heading: 'text-2xl font-bold',
          subheading: 'text-lg font-semibold',
          body: 'text-base',
          caption: 'text-sm',
        };
    }
  };

  return getFontSizes();
}
