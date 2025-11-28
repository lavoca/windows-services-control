<template>
  
  <div class="min-h-screen bg-[#212121] text-gray-200 p-6">
    <h1 class="text-3xl font-bold mb-6 text-blue-400">Windows Services</h1>

    <div class="flex items-center gap-4 mb-6">
      <button
        @click="loadServices"
        class="px-4 py-2 rounded bg-blue-600 hover:bg-blue-700 transition"
      >
        Refresh Services
      </button>

      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search services..."
        class="flex-1 px-4 py-2 bg-[#1a1a1a] border border-gray-700 rounded focus:border-blue-500 focus:outline-none"
      />
    </div>

    <div v-if="loading" class="text-gray-400">Loading...</div>
    <div v-if="error" class="mb-4 p-4 bg-red-900/50 border border-red-600 rounded">
      {{ error }}
      <button @click="error = null" class="ml-4 underline">Dismiss</button>
    </div>

    <div class="mb-4 text-gray-400 text-sm">
      Showing {{ filteredServices.length }} of {{ services.length }} services
    </div>

    <div class="grid gap-6">
      <ServiceCard
        v-for="service in filteredServices"
        :key="service.name"
        :service="service"
        @start="start"
        @stop="stop"
        @pause="pause"
        @resume="resume"
        @searchServiceSafety="searchServiceSafety"
      />
    </div>

    <div v-if="filteredServices.length === 0 && !loading" class="text-center text-gray-400 mt-8">
      No services found matching "{{ searchQuery }}"
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ServiceCard from "./components/ServiceCard.vue";
import { open } from "@tauri-apps/plugin-shell";



interface ServiceInfo {
  name: string;
  display_name: string;
  status: string;
  service_type: string;
  can_interact: boolean;
}

const services = ref<ServiceInfo[]>([]);
const loading = ref(false);
// Reactive state: String to store error messages (null if no error)
const error = ref<string | null>(null);
// Reactive state: String to store the user's search query
const searchQuery = ref('');


const filteredServices = computed(() => {
  if(!searchQuery.value) return services.value;
  const query = searchQuery.value.toLowerCase();
  
  return services.value.filter(service => 
    // filter the services array for ones that match the name or display name queried 
    service.name.toLowerCase().includes(query) || 
    service.display_name.toLowerCase().includes(query)
  )
})



async function loadServices() {
  loading.value = true;
  error.value = null;
  try {
    services.value = await invoke("enumerate_services");
    console.log(services.value)
  } catch (e) {
    error.value = `failed loading services: ${e}`;
  }finally {
    loading.value = false;
  }
}

async function start(name: string) {
  error.value = null;
  try{
    await invoke("start_service", { serviceName: name });
    await loadServices();
  } catch (e) {
    error.value = `problem starting ${name}: ${e}`;
  }
  
}

async function stop(name: string) {
  error.value = null;
  try {
    await invoke("stop_service", { serviceName: name });
    await loadServices();
  } catch (e) {
    error.value = `problem stoping ${name}: ${e}`;
  }
  
}

async function pause(name: string) {
  error.value = null;
  try {
    await invoke("pause_service", { serviceName: name });
    await loadServices();
  } catch(e) {
    error.value = `problem pausing ${name}: ${e}`;
  }

}

async function resume(name: string) {
  error.value = null;
  try {
    await invoke("resume_service", { serviceName: name });
    await loadServices();
  } catch (e) {
    error.value = `problem resuming ${name}: ${e}`
  }
  
}

async function searchServiceSafety(name: string) {
  const url =(`https://google.com/search?q=${encodeURIComponent(`is ${name} windows service safe to stop`)}`); 
  await open(url);
}

onMounted(loadServices);
</script>
