<script setup lang="ts">
import { AvatarFallback, useForwardProps } from "reka-ui";
import type { AvatarFallbackProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends AvatarFallbackProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AvatarFallback
    v-bind="forwarded"
    :class="
      cn(
        'flex h-full w-full items-center justify-center rounded-full bg-muted',
        props.class
      )
    "
  >
    <slot />
  </AvatarFallback>
</template>
