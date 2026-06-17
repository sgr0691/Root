import React from "react";
import { useCurrentFrame, useVideoConfig, spring, interpolate } from "remotion";

interface LogoProps {
  delay?: number;
  size?: "small" | "medium" | "large";
}

export const Logo: React.FC<LogoProps> = ({ delay = 0, size = "medium" }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const progress = spring({
    frame: frame - delay,
    fps,
    config: { damping: 20, stiffness: 100, mass: 0.5 },
  });

  const opacity = progress;
  const scale = interpolate(progress, [0, 1], [0.9, 1]);

  const sizes = {
    small: { width: 80, height: 35 },
    medium: { width: 120, height: 53 },
    large: { width: 180, height: 79 },
  };

  const { width, height } = sizes[size];

  return (
    <div
      style={{
        opacity,
        transform: `scale(${scale})`,
      }}
    >
      <svg
        width={width}
        height={height}
        viewBox="0 0 123 54"
        fill="none"
        xmlns="http://www.w3.org/2000/svg"
      >
        <g filter="url(#filter0_ddiii_137_2869)">
          <g clipPath="url(#clip0_137_2869)">
            <rect x="3" width="48" height="48" rx="12" fill="#D920AE" />
            <rect
              width="48"
              height="48"
              transform="translate(3)"
              fill="url(#paint0_linear_137_2869)"
            />
            <g filter="url(#filter1_d_137_2869)">
              <path
                d="M29 29C34.5228 29 39 24.5228 39 19C39 13.4772 34.5228 9 29 9H15V29H29Z"
                fill="url(#paint1_linear_137_2869)"
              />
              <path
                d="M15 9L39 39H23L15 29V9Z"
                fill="url(#paint2_linear_137_2869)"
              />
              <path
                d="M30.8616 28.827C30.2585 28.9406 29.6362 29 29 29H15V9L30.8616 28.827Z"
                fill="url(#paint3_linear_137_2869)"
              />
            </g>
          </g>
          <rect
            x="4"
            y="1"
            width="46"
            height="46"
            rx="11"
            stroke="url(#paint4_linear_137_2869)"
            strokeWidth="2"
          />
        </g>
        <g opacity="0.84">
          <path
            d="M63 34V13H71.61C73.81 13 75.52 13.63 76.74 14.89C77.96 16.15 78.57 17.72 78.57 19.6C78.57 21.06 78.23 22.33 77.55 23.41C76.89 24.49 75.93 25.28 74.67 25.78L78.87 33.7V34H74.55L70.65 26.38H66.9V34H63ZM66.9 22.66H71.37C72.33 22.66 73.1 22.42 73.68 21.94C74.28 21.44 74.58 20.66 74.58 19.6C74.58 18.6 74.28 17.87 73.68 17.41C73.1 16.93 72.33 16.69 71.37 16.69H66.9V22.66Z"
            fill="white"
          />
          <path
            d="M92.6372 32.14C91.3772 33.56 89.6472 34.27 87.4472 34.27C85.2472 34.27 83.5172 33.56 82.2572 32.14C80.9972 30.72 80.3672 29 80.3672 26.98V25.33C80.3672 23.31 80.9972 21.59 82.2572 20.17C83.5172 18.75 85.2472 18.04 87.4472 18.04C89.6472 18.04 91.3772 18.75 92.6372 20.17C93.8972 21.59 94.5272 23.31 94.5272 25.33V26.98C94.5272 29 93.8972 30.72 92.6372 32.14ZM84.1172 26.98C84.1172 28.04 84.3972 28.93 84.9572 29.65C85.5372 30.35 86.3672 30.7 87.4472 30.7C88.5272 30.7 89.3472 30.35 89.9072 29.65C90.4872 28.93 90.7772 28.04 90.7772 26.98V25.33C90.7772 24.27 90.4872 23.39 89.9072 22.69C89.3472 21.97 88.5272 21.61 87.4472 21.61C86.3672 21.61 85.5372 21.97 84.9572 22.69C84.3972 23.39 84.1172 24.27 84.1172 25.33V26.98Z"
            fill="white"
          />
          <path
            d="M108.604 32.14C107.344 33.56 105.614 34.27 103.414 34.27C101.214 34.27 99.484 33.56 98.224 32.14C96.964 30.72 96.334 29 96.334 26.98V25.33C96.334 23.31 96.964 21.59 98.224 20.17C99.484 18.75 101.214 18.04 103.414 18.04C105.614 18.04 107.344 18.75 108.604 20.17C109.864 21.59 110.494 23.31 110.494 25.33V26.98C110.494 29 109.864 30.72 108.604 32.14ZM100.084 26.98C100.084 28.04 100.364 28.93 100.924 29.65C101.504 30.35 102.334 30.7 103.414 30.7C104.494 30.7 105.314 30.35 105.874 29.65C106.454 28.93 106.744 28.04 106.744 26.98V25.33C106.744 24.27 106.454 23.39 105.874 22.69C105.314 21.97 104.494 21.61 103.414 21.61C102.334 21.61 101.504 21.97 100.924 22.69C100.364 23.39 100.084 24.27 100.084 25.33V26.98Z"
            fill="white"
          />
          <path
            d="M118.241 34C117.041 34 116.091 33.63 115.391 32.89C114.711 32.15 114.371 31.14 114.371 29.86V21.76H112.001V18.31H114.491V14.71H118.121V18.31H121.421V21.76H118.121V29.14C118.121 29.66 118.221 30.03 118.421 30.25C118.621 30.45 118.981 30.55 119.501 30.55H122.081V34H118.241Z"
            fill="white"
          />
        </g>
        <defs>
          <filter
            id="filter0_ddiii_137_2869"
            x="0"
            y="-3"
            width="54"
            height="57"
            filterUnits="userSpaceOnUse"
            colorInterpolationFilters="sRGB"
          >
            <feFlood floodOpacity="0" result="BackgroundImageFix" />
            <feColorMatrix
              in="SourceAlpha"
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
              result="hardAlpha"
            />
            <feOffset dy="1" />
            <feGaussianBlur stdDeviation="0.5" />
            <feComposite in2="hardAlpha" operator="out" />
            <feColorMatrix
              type="matrix"
              values="0 0 0 0 0.162923 0 0 0 0 0.162923 0 0 0 0 0.162923 0 0 0 0.08 0"
            />
            <feBlend
              mode="normal"
              in2="BackgroundImageFix"
              result="effect1_dropShadow_137_2869"
            />
            <feColorMatrix
              in="SourceAlpha"
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
              result="hardAlpha"
            />
            <feMorphology
              radius="1"
              operator="erode"
              in="SourceAlpha"
              result="effect2_dropShadow_137_2869"
            />
            <feOffset dy="3" />
            <feGaussianBlur stdDeviation="2" />
            <feComposite in2="hardAlpha" operator="out" />
            <feColorMatrix
              type="matrix"
              values="0 0 0 0 0.164706 0 0 0 0 0.164706 0 0 0 0 0.164706 0 0 0 0.14 0"
            />
            <feBlend
              mode="normal"
              in2="effect1_dropShadow_137_2869"
              result="effect2_dropShadow_137_2869"
            />
            <feBlend mode="normal" in="SourceGraphic" in2="effect2_dropShadow_137_2869" result="shape" />
            <feColorMatrix
              in="SourceAlpha"
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
              result="hardAlpha"
            />
            <feOffset dy="-3" />
            <feGaussianBlur stdDeviation="1.5" />
            <feComposite in2="hardAlpha" operator="arithmetic" k2="-1" k3="1" />
            <feColorMatrix
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.1 0"
            />
            <feBlend
              mode="normal"
              in2="shape"
              result="effect3_innerShadow_137_2869"
            />
            <feColorMatrix
              in="SourceAlpha"
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
              result="hardAlpha"
            />
            <feOffset dy="3" />
            <feGaussianBlur stdDeviation="1.5" />
            <feComposite in2="hardAlpha" operator="arithmetic" k2="-1" k3="1" />
            <feColorMatrix
              type="matrix"
              values="0 0 0 0 1 0 0 0 0 1 0 0 0 0 1 0 0 0 0.1 0"
            />
            <feBlend
              mode="normal"
              in2="effect3_innerShadow_137_2869"
              result="effect4_innerShadow_137_2869"
            />
            <feColorMatrix
              in="SourceAlpha"
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
              result="hardAlpha"
            />
            <feMorphology
              radius="1"
              operator="erode"
              in="SourceAlpha"
              result="effect5_innerShadow_137_2869"
            />
            <feOffset />
            <feComposite in2="hardAlpha" operator="arithmetic" k2="-1" k3="1" />
            <feColorMatrix
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0.2 0"
            />
            <feBlend
              mode="normal"
              in2="effect4_innerShadow_137_2869"
              result="effect5_innerShadow_137_2869"
            />
          </filter>
          <filter
            id="filter1_d_137_2869"
            x="12"
            y="5.25"
            width="30"
            height="42"
            filterUnits="userSpaceOnUse"
            colorInterpolationFilters="sRGB"
          >
            <feFlood floodOpacity="0" result="BackgroundImageFix" />
            <feColorMatrix
              in="SourceAlpha"
              type="matrix"
              values="0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 127 0"
              result="hardAlpha"
            />
            <feMorphology
              radius="1.5"
              operator="erode"
              in="SourceAlpha"
              result="effect1_dropShadow_137_2869"
            />
            <feOffset dy="2.25" />
            <feGaussianBlur stdDeviation="2.25" />
            <feComposite in2="hardAlpha" operator="out" />
            <feColorMatrix
              type="matrix"
              values="0 0 0 0 0.141176 0 0 0 0 0.141176 0 0 0 0 0.141176 0 0 0 0.1 0"
            />
            <feBlend
              mode="normal"
              in2="BackgroundImageFix"
              result="effect1_dropShadow_137_2869"
            />
            <feBlend
              mode="normal"
              in="SourceGraphic"
              in2="effect1_dropShadow_137_2869"
              result="shape"
            />
          </filter>
          <linearGradient
            id="paint0_linear_137_2869"
            x1="24"
            y1="5.96047e-07"
            x2="26"
            y2="48"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="white" stopOpacity="0" />
            <stop offset="1" stopColor="white" stopOpacity="0.12" />
          </linearGradient>
          <linearGradient
            id="paint1_linear_137_2869"
            x1="27"
            y1="9"
            x2="27"
            y2="29"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="white" stopOpacity="0.8" />
            <stop offset="1" stopColor="white" stopOpacity="0.5" />
          </linearGradient>
          <linearGradient
            id="paint2_linear_137_2869"
            x1="27"
            y1="9"
            x2="27"
            y2="39"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="white" stopOpacity="0.8" />
            <stop offset="1" stopColor="white" stopOpacity="0.5" />
          </linearGradient>
          <linearGradient
            id="paint3_linear_137_2869"
            x1="22.9308"
            y1="9"
            x2="22.9308"
            y2="29"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="white" stopOpacity="0.8" />
            <stop offset="1" stopColor="white" stopOpacity="0.5" />
          </linearGradient>
          <linearGradient
            id="paint4_linear_137_2869"
            x1="27"
            y1="0"
            x2="27"
            y2="48"
            gradientUnits="userSpaceOnUse"
          >
            <stop stopColor="white" stopOpacity="0.12" />
            <stop offset="1" stopColor="white" stopOpacity="0" />
          </linearGradient>
          <clipPath id="clip0_137_2869">
            <rect x="3" width="48" height="48" rx="12" fill="white" />
          </clipPath>
        </defs>
      </svg>
    </div>
  );
};
