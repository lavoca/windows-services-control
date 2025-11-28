<template>
  <div
    class="bg-[#1f1f1f]/70 backdrop-blur-md rounded-2xl border border-white/10
           p-5 shadow-lg hover:shadow-2xl transition-transform duration-150 hover:scale-[1.01]"
  >
    <div class="flex justify-between items-start">
      <!-- Left: service info -->
      <div class="flex-1">
        <h2 class="text-xl font-semibold text-gray-100 tracking-wide">{{ service.display_name }}</h2>
        <p class="text-gray-400 text-sm">{{ service.name }}</p>

        <p class="mt-3 text-sm">
          <span class="text-gray-500">Status: </span>
          <span
            :class="{
              'text-green-400': service.status === 'Running',
              'text-yellow-400': service.status.includes('Pending'),
              'text-red-400': service.status === 'Stopped'
            }"
            class="font-medium"
          >
            {{ service.status }}
          </span>
        </p>

        <p class="text-gray-500 text-xs mt-1">{{ service.service_type }}</p>
      </div>


      <!-- Right: buttons (only if can_interact) -->
      <div v-if="service.can_interact" class="flex flex-col gap-2 ml-4">
        <button
          v-if="service.status === 'Stopped' || service.status === 'Paused'"
          @click="$emit('start', service.name)"
          class="px-3 py-1.5 rounded-xl text-sm font-medium border border-white/10
                 bg-green-600 text-white hover:bg-green-600/30 transition"
        >
          Start
        </button>

        <button
          v-if="service.status === 'Running'"
          @click="$emit('stop', service.name)"
          class="px-3 py-1.5 rounded-xl text-sm font-medium border border-white/10
                 bg-red-600/20 text-red-400 hover:bg-red-600/30 transition"
        >
          Stop
        </button>

        <button
          v-if="service.status === 'Running'"
          @click="$emit('pause', service.name)"
          class="px-3 py-1.5 rounded-xl text-sm font-medium border border-white/10
                 bg-yellow-600/20 text-yellow-400 hover:bg-yellow-600/30 transition"
        >
          Pause
        </button>

        <button
          v-if="service.status === 'Paused'"
          @click="$emit('resume', service.name)"
          class="px-3 py-1.5 rounded-xl text-sm font-medium border border-white/10
                 bg-blue-600/20 text-blue-400 hover:bg-blue-600/30 transition"
        >
          Resume
        </button>
      </div>
                 
    </div>
        <!-- Google search button -->
    <button
      @click="$emit('searchServiceSafety', service.name)"
      class="mt-4 w-full px-4 py-2 rounded-xl bg-blue-600/20 text-blue-400 border border-blue-500/30 hover:bg-blue-600/30 transition flex items-center justify-center gap-2"
    >
      🔍 Research this service
    </button>
  </div>
</template>

<script setup lang="ts">


defineProps({
  service: { type: Object, required: true }
})

defineEmits(["start", "stop", "pause", "resume", "searchServiceSafety"])
</script>
