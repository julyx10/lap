<template>

  <div class="w-screen h-screen flex flex-col border border-gray-800 rounded-lg overflow-hidden">
    <!-- Title Bar -->
    <TitleBar :titlebar="$t('settings')" :resizable="false" :hasSearch="false"/>

    <!-- Main Content -->
    <div class="flex-1 flex p-4 t-color-bg t-color-text overflow-auto t-scrollbar-dark">

      <div class="w-32">
        pane
      </div>
      
      <div class="flex-grow">

        <!-- General Settings Section -->
        <section>
          <span class="text-lg font-bold mb-2">General</span>

          <!-- Toggle for Dark Mode -->
          <div class="flex items-center justify-between mb-4">
            <label for="dark-mode" >Dark Mode</label>
            <input
              type="checkbox" 
              id="dark-mode" 
              v-model="darkMode" 
            />
          </div>

          <!-- Input for Username -->
          <div class="mb-4">
            <label for="username" class="block text-lg">Username</label>
            <input 
              type="text" 
              id="username" 
              v-model="username" 
              class="px-2 py-1 w-full text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
              placeholder="Enter your username" 
            />
          </div>
        </section>

        <!-- Notifications Settings Section -->
        <section>
          <h2 class="text-xl font-semibold mb-2">Notifications</h2>

          <!-- Toggle for Email Notifications -->
          <div class="flex items-center justify-between mb-4">
            <label for="email-notifications" class="text-lg">Email Notifications</label>
            <input type="checkbox" id="email-notifications" v-model="emailNotifications" />
          </div>

          <!-- Select Notification Frequency -->
          <div class="mb-4">
            <label for="notification-frequency" class="block text-lg">Notification Frequency</label>
            <select 
              id="notification-frequency" 
              v-model="notificationFrequency" 
              class="px-2 py-1 w-full text-sm border rounded-md t-input-color-bg t-color-border t-input-focus"
            >
              <option value="none">None</option>
              <option value="daily">Daily</option>
              <option value="weekly">Weekly</option>
            </select>
          </div>
        </section>

        <!-- Save Button -->
        <div class="text-right">
          <button @click="saveSettings" class="bg-blue-500 text-white px-4 py-2 rounded">Save</button>
        </div>
      </div>

    </div>  

  </div>
  
</template>

<script setup>

import TitleBar from '@/components/TitleBar.vue';


</script>