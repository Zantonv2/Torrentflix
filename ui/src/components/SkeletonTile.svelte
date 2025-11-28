<script lang="ts">
  export let posterHeight = 400;
</script>

<div style="width: 100%; flex-shrink: 0;">
  <div class="bg-netflix-card-bg rounded-xl overflow-hidden cursor-pointer transition-all duration-300 group block w-full relative animate-pulse">
    <!-- Poster skeleton -->
    <div 
      class="relative bg-netflix-card-hover w-full shimmer"
      style="height: {posterHeight}px;"
    >
      <!-- Year badge placeholder skeleton -->
      <div class="absolute top-2 right-2 bg-black/60 backdrop-blur-sm text-white text-xs px-2 py-1 rounded-md font-medium z-10 shimmer w-12 h-5"></div>
    </div>
    
    <!-- Info skeleton -->
    <div class="p-3 flex flex-col" style="min-height: 80px; max-height: 80px;">
      <!-- Title lines -->
      <div class="h-4 bg-netflix-card-hover rounded shimmer w-full mb-2"></div>
      <div class="h-4 bg-netflix-card-hover rounded shimmer w-3/4 mb-1"></div>
      
      <!-- Quality line -->
      <div class="h-3 bg-netflix-card-hover rounded shimmer w-1/2 mb-1"></div>
      
      <!-- Torrent info line -->
      <div class="h-3 bg-netflix-card-hover rounded shimmer w-2/3"></div>
    </div>
  </div>
</div>

<style>
  @keyframes shimmer {
    0% {
      background-position: -200px 0;
    }
    100% {
      background-position: 200px 0;
    }
  }
  
  .shimmer {
    background: linear-gradient(
      90deg,
      #1a1a1a 0%,
      #2a2a2a 50%,
      #1a1a1a 100%
    );
    background-size: 200px 100%;
    animation: shimmer 1.5s infinite;
  }
  
  :global(.bg-netflix-card-bg) {
    background-color: #1a1a1a;
  }
  
  :global(.bg-netflix-card-hover) {
    background-color: #2a2a2a;
  }
</style>
