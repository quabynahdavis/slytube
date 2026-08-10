<script setup lang="ts">
import { TabsContent, useForwardProps } from "reka-ui";
import type { TabsContentProps } from "reka-ui";
import { cn } from "@/lib/utils";

interface Props extends TabsContentProps {
  class?: string;
}

const props = defineProps<Props>();
const forwarded = useForwardProps(props);
</script>

<template>
  <TabsContent
    v-bind="forwarded"
    :class="
      cn(
        'mt-2 ring-offset-background focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2',
        props.class
      )
    "
  >
    <slot />
  </TabsContent>
</template>
