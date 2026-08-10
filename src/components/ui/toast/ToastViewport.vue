<script setup lang="ts">
import { ToastViewport, useForwardProps } from "reka-ui";
import type { ToastViewportProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ToastViewportProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ToastViewport
    v-bind="forwarded"
    :class="
      cn(
        'fixed top-0 z-[100] flex max-h-screen w-full flex-col-reverse p-4 sm:bottom-0 sm:right-0 sm:top-auto sm:flex-col md:max-w-[420px]',
        props.class
      )
    "
  />
</template>
