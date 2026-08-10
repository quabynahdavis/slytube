<script setup lang="ts">
import { DropdownMenuItem, useForwardProps } from "reka-ui";
import type { DropdownMenuItemProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends DropdownMenuItemProps {
  class?: string;
  inset?: boolean;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <DropdownMenuItem
    v-bind="forwarded"
    :class="
      cn(
        'relative flex cursor-default select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors focus:bg-accent focus:text-accent-foreground data-[disabled]:pointer-events-none data-[disabled]:opacity-50',
        inset && 'pl-8',
        props.class
      )
    "
  >
    <slot />
  </DropdownMenuItem>
</template>
