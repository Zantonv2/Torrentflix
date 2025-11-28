<script>
  // Simple loading dots component for pagination feedback
</script>

<div class="flex justify-center items-center py-4">
  <div class="flex space-x-2">
    <div class="w-2 h-2 bg-netflix-red rounded-full animate-bounce" style="animation-delay: 0s;"></div>
    <div class="w-2 h-2 bg-netflix-red rounded-full animate-bounce" style="animation-delay: 0.2s;"></div>
    <div class="w-2 h-2 bg-netflix-red rounded-full animate-bounce" style="animation-delay: 0.4s;"></div>
  </div>
</div>

<style>
  @keyframes bounce {
    0%, 80%, 100% {
      transform: scale(0);
      opacity: 0.5;
    }
    40% {
      transform: scale(1);
      opacity: 1;
    }
  }
  
  .animate-bounce {
    animation: bounce 1.4s infinite ease-in-out both;
  }
</style>
