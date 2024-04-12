<script setup lang="ts">
import { useRouter } from "vue-router";

const router = useRouter();
</script>

<template>
  <div class="navbar bg-base-100">
    <div class="navbar-start">
      <div class="dropdown">
        <div tabindex="0" role="button" class="btn btn-ghost btn-circle">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            class="h-5 w-5"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M4 6h16M4 12h16M4 18h7"
            />
          </svg>
        </div>
        <ul
          tabindex="0"
          class="menu menu-sm dropdown-content mt-3 z-[1] p-2 shadow bg-base-100 rounded-box w-52"
        >
          <li><router-link to="/">Home</router-link></li>
          <li><router-link to="/assets">My Assets</router-link></li>
          <li>
            <router-link to="/settings">Settings</router-link>
          </li>
          <li><router-link to="/about">About</router-link></li>
        </ul>
      </div>
    </div>
    <div class="navbar-center">
      <button class="btn btn-ghost text-xl" @click="router.push('/')">
        Checked
      </button>
    </div>
    <div class="navbar-end">
      <router-link to="/search" class="px-2">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          class="h-5 w-5"
          fill="none"
          viewBox="0 0 24 24"
          stroke="currentColor"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
      </router-link>
    </div>
  </div>
</template>
