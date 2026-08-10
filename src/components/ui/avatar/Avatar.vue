<script setup lang="ts">
import { AvatarRoot, useForwardProps } from "reka-ui";
import type { AvatarRootProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends AvatarRootProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <AvatarRoot
    v-bind="forwarded"
    :class="
      cn(
        'relative flex h-10 w-10 shrink-0 overflow-hidden rounded-full',
        props.class
      )
    "
  >
    <slot />
  </AvatarRoot>
</template>
