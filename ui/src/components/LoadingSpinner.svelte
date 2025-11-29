<script lang="ts">
  // LoadingSpinner component - no props needed
</script>

<div style="display: flex; flex-direction: column; align-items: center; justify-content: center; min-height: 100vh; background-color: #000000; position: fixed; top: 0; left: 0; width: 100%; z-index: 9999;">
  <div style="width: 64px; height: 64px; border: 4px solid #374151; border-top: 4px solid #e50914; border-radius: 50%; animation: spin 1s linear infinite;"></div>
  <div style="margin-top: 24px; color: #9ca3af; font-size: 18px; animation: pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite; font-weight: 500;">Loading movies...</div>
  <div style="margin-top: 8px; color: #6b7280; font-size: 14px;">Fetching latest from Monna2</div>
</div>

<style>
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
  
  @keyframes pulse {
    50%, 100% {
      opacity: 1;
    }
    25% {
      opacity: .5;
    }
  }
</style>
