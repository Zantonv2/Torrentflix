<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';

  interface FileVersion {
    id: number;
    media_item_id: number;
    file_size: number;
    status: string;
    is_preferred: boolean;
    resolution_width?: number;
    resolution_height?: number;
    video_codec?: string;
    audio_codec?: string;
    quality_label?: string;
    source_type?: string;
    added_at: string;
    fast_hash?: string;
    full_hash?: string;
  }

  interface QualityScore {
    total: number;
    resolution_score: number;
    codec_score: number;
    source_score: number;
    size_score: number;
  }

  export let versions: FileVersion[] = [];
  export let mediaId: number;
  export let onVersionSelected: (version: FileVersion) => void = () => {};

  let selectedVersionId: number | null = null;
  let qualityScores: Map<number, QualityScore> = new Map();
  let isLoading = false;
  let error: string | null = null;

  async function setPreferredVersion(versionId: number) {
    isLoading = true;
    error = null;

    try {
      await invoke('set_preferred_version', {
        version_id: versionId,
      });

      // Update local state
      versions = versions.map((v) => ({
        ...v,
        is_preferred: v.id === versionId,
      }));
    } catch (err) {
      error = `Failed to set preferred version: ${err}`;
      console.error('Error:', err);
    } finally {
      isLoading = false;
    }
  }

  function getResolutionLabel(width?: number, height?: number): string {
    if (!height) return 'Unknown';
    if (height >= 2160) return '4K';
    if (height >= 1080) return '1080p';
    if (height >= 720) return '720p';
    return 'SD';
  }

  function getResolutionScore(height?: number): number {
    if (!height) return 0;
    if (height >= 2160) return 100;
    if (height >= 1080) return 80;
    if (height >= 720) return 60;
    return 40;
  }

  function getCodecScore(codec?: string): number {
    if (!codec) return 0;
    const lower = codec.toLowerCase();
    if (lower.includes('h.265') || lower.includes('hevc')) return 100;
    if (lower.includes('h.264') || lower.includes('avc')) return 80;
    return 50;
  }

  function getSourceScore(source?: string): number {
    if (!source) return 0;
    const lower = source.toLowerCase();
    if (lower.includes('bluray')) return 100;
    if (lower.includes('web-dl') || lower.includes('webdl')) return 80;
    if (lower.includes('hdtv')) return 60;
    if (lower.includes('dvd')) return 40;
    return 20;
  }

  function getSizeScore(sizeBytes: number): number {
    const sizeGB = sizeBytes / (1024 * 1024 * 1024);
    // Prefer sizes between 2-8 GB
    if (sizeGB >= 2 && sizeGB <= 8) return 100;
    if (sizeGB >= 1 && sizeGB <= 12) return 80;
    if (sizeGB >= 0.5 && sizeGB <= 15) return 60;
    return 40;
  }

  function calculateQualityScore(version: FileVersion): QualityScore {
    const resScore = getResolutionScore(version.resolution_height);
    const codecScore = getCodecScore(version.video_codec);
    const sourceScore = getSourceScore(version.source_type);
    const sizeScore = getSizeScore(version.file_size);

    const total = (resScore * 0.4 + codecScore * 0.3 + sourceScore * 0.2 + sizeScore * 0.1);

    return {
      total: Math.round(total),
      resolution_score: resScore,
      codec_score: codecScore,
      source_score: sourceScore,
      size_score: sizeScore,
    };
  }

  function getScoreColor(score: number): string {
    if (score >= 80) return 'text-green-400';
    if (score >= 60) return 'text-yellow-400';
    if (score >= 40) return 'text-orange-400';
    return 'text-red-400';
  }

  function getScoreBgColor(score: number): string {
    if (score >= 80) return 'bg-green-900';
    if (score >= 60) return 'bg-yellow-900';
    if (score >= 40) return 'bg-orange-900';
    return 'bg-red-900';
  }

  $: {
    qualityScores.clear();
    versions.forEach((v) => {
      qualityScores.set(v.id, calculateQualityScore(v));
    });
    qualityScores = qualityScores;
  }

  const sortedVersions = () => {
    return [...versions].sort((a, b) => {
      const scoreA = qualityScores.get(a.id)?.total || 0;
      const scoreB = qualityScores.get(b.id)?.total || 0;
      return scoreB - scoreA;
    });
  };
</script>

<div class="version-manager w-full text-white">
  {#if error}
    <div class="bg-red-900 border border-red-700 text-red-100 px-4 py-3 rounded mb-4">
      {error}
    </div>
  {/if}

  {#if versions.length === 0}
    <div class="text-center py-8 text-gray-400">
      <p>No file versions available</p>
    </div>
  {:else}
    <div class="space-y-4">
      <!-- Summary -->
      <div class="bg-gray-700 p-4 rounded">
        <h3 class="font-semibold mb-2">Version Summary</h3>
        <div class="grid grid-cols-2 sm:grid-cols-4 gap-4 text-sm">
          <div>
            <p class="text-gray-400">Total Versions</p>
            <p class="text-lg font-semibold">{versions.length}</p>
          </div>
          <div>
            <p class="text-gray-400">Present</p>
            <p class="text-lg font-semibold text-green-400">
              {versions.filter((v) => v.status === 'present').length}
            </p>
          </div>
          <div>
            <p class="text-gray-400">Missing</p>
            <p class="text-lg font-semibold text-red-400">
              {versions.filter((v) => v.status === 'missing').length}
            </p>
          </div>
          <div>
            <p class="text-gray-400">Total Size</p>
            <p class="text-lg font-semibold">
              {(versions.reduce((sum, v) => sum + v.file_size, 0) / (1024 * 1024 * 1024)).toFixed(2)} GB
            </p>
          </div>
        </div>
      </div>

      <!-- Versions List -->
      <div class="space-y-3">
        {#each sortedVersions() as version (version.id)}
          {@const score = qualityScores.get(version.id)}
          <div
            class={`p-4 rounded border-2 cursor-pointer transition ${
              selectedVersionId === version.id
                ? 'border-blue-500 bg-gray-700'
                : 'border-gray-600 bg-gray-800 hover:border-gray-500'
            }`}
            on:click={() => {
              selectedVersionId = version.id;
              onVersionSelected(version);
            }}
          >
            <!-- Header -->
            <div class="flex justify-between items-start mb-3">
              <div class="flex-1">
                <div class="flex items-center gap-2 mb-1">
                  <h4 class="font-semibold">
                    {getResolutionLabel(version.resolution_width, version.resolution_height)}
                    {#if version.quality_label}
                      - {version.quality_label}
                    {/if}
                  </h4>
                  {#if version.is_preferred}
                    <span class="bg-yellow-600 px-2 py-0.5 rounded text-xs font-semibold">
                      ⭐ Preferred
                    </span>
                  {/if}
                </div>
                <p class="text-sm text-gray-400">
                  {(version.file_size / (1024 * 1024 * 1024)).toFixed(2)} GB
                  • Added {new Date(version.added_at).toLocaleDateString()}
                </p>
              </div>

              <!-- Quality Score -->
              {#if score}
                <div class={`text-center px-3 py-2 rounded ${getScoreBgColor(score.total)}`}>
                  <p class={`text-2xl font-bold ${getScoreColor(score.total)}`}>
                    {score.total}
                  </p>
                  <p class="text-xs text-gray-300">Quality</p>
                </div>
              {/if}
            </div>

            <!-- Status Badge -->
            <div class="mb-3">
              <span class={`px-2 py-1 rounded text-xs font-semibold ${
                version.status === 'present'
                  ? 'bg-green-900 text-green-200'
                  : version.status === 'missing'
                    ? 'bg-red-900 text-red-200'
                    : 'bg-gray-600 text-gray-200'
              }`}>
                {version.status}
              </span>
            </div>

            <!-- Technical Details -->
            <div class="grid grid-cols-2 gap-2 text-sm mb-3">
              {#if version.video_codec}
                <div>
                  <p class="text-gray-400">Video Codec</p>
                  <p class="font-semibold">{version.video_codec}</p>
                </div>
              {/if}
              {#if version.audio_codec}
                <div>
                  <p class="text-gray-400">Audio Codec</p>
                  <p class="font-semibold">{version.audio_codec}</p>
                </div>
              {/if}
              {#if version.source_type}
                <div>
                  <p class="text-gray-400">Source</p>
                  <p class="font-semibold">{version.source_type}</p>
                </div>
              {/if}
              {#if version.resolution_width && version.resolution_height}
                <div>
                  <p class="text-gray-400">Resolution</p>
                  <p class="font-semibold">{version.resolution_width}x{version.resolution_height}</p>
                </div>
              {/if}
            </div>

            <!-- Score Breakdown -->
            {#if score}
              <div class="bg-gray-700 p-2 rounded mb-3 text-xs">
                <div class="grid grid-cols-4 gap-2">
                  <div>
                    <p class="text-gray-400">Resolution</p>
                    <p class={`font-semibold ${getScoreColor(score.resolution_score)}`}>
                      {score.resolution_score}
                    </p>
                  </div>
                  <div>
                    <p class="text-gray-400">Codec</p>
                    <p class={`font-semibold ${getScoreColor(score.codec_score)}`}>
                      {score.codec_score}
                    </p>
                  </div>
                  <div>
                    <p class="text-gray-400">Source</p>
                    <p class={`font-semibold ${getScoreColor(score.source_score)}`}>
                      {score.source_score}
                    </p>
                  </div>
                  <div>
                    <p class="text-gray-400">Size</p>
                    <p class={`font-semibold ${getScoreColor(score.size_score)}`}>
                      {score.size_score}
                    </p>
                  </div>
                </div>
              </div>
            {/if}

            <!-- Actions -->
            {#if version.status === 'present'}
              <button
                on:click={(e) => {
                  e.stopPropagation();
                  setPreferredVersion(version.id);
                }}
                disabled={isLoading}
                class={`w-full px-3 py-2 rounded text-sm font-semibold transition ${
                  version.is_preferred
                    ? 'bg-yellow-600 hover:bg-yellow-700'
                    : 'bg-blue-600 hover:bg-blue-700'
                } disabled:opacity-50 disabled:cursor-not-allowed`}
              >
                {version.is_preferred ? '⭐ Preferred' : 'Set as Preferred'}
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<style>
  .version-manager {
    display: flex;
    flex-direction: column;
  }
</style>
