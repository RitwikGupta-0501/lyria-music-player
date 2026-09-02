<script>
  import '../app.css';
</script>



<!-- Global Reusable SVG Filters for Physical Optical Refraction & Glass Effects -->
<svg
  id="echo-glass-filters"
  aria-hidden="true"
  width="0"
  height="0"
  style="position: absolute; width: 0; height: 0; overflow: hidden; pointer-events: none; border: none;"
>
  <defs>
    <!-- Standard Physical Refraction for Panels, Floating Bars, and Cards -->
    <filter id="glass-refraction" x="0%" y="0%" width="100%" height="100%">
      <feTurbulence
        type="fractalNoise"
        baseFrequency="0.04 0.04"
        numOctaves="2"
        result="noise"
      />
      <feDisplacementMap
        in="SourceGraphic"
        in2="noise"
        scale="10"
        xChannelSelector="R"
        yChannelSelector="G"
        result="displaced"
      />
    </filter>

    <!-- Subtle Refraction for Buttons, Pills, and Small Controls -->
    <filter id="glass-refraction-subtle" x="0%" y="0%" width="100%" height="100%">
      <feTurbulence
        type="fractalNoise"
        baseFrequency="0.08 0.08"
        numOctaves="1"
        result="noise"
      />
      <feDisplacementMap
        in="SourceGraphic"
        in2="noise"
        scale="4"
        xChannelSelector="R"
        yChannelSelector="G"
        result="displaced"
      />
    </filter>
  </defs>
</svg>

<slot />
