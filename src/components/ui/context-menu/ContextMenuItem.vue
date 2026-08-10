<script setup lang="ts">
import { ContextMenuItem, useForwardProps } from "reka-ui";
import type { ContextMenuItemProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends ContextMenuItemProps {
  class?: string;
  inset?: boolean;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <ContextMenuItem
    v-bind="forwarded"
    :class="
      cn(
        'relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        inset && 'pl-8',
        props.class
      )
    "
  >
    <slot />
  </ContextMenuItem>
</template>
