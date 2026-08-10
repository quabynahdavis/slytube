<script setup lang="ts">
import { TabsRoot, useForwardPropsEmits } from "reka-ui";
import type { TabsRootEmits, TabsRootProps } from "reka-ui";

interface Props extends TabsRootProps {}

const props = defineProps<Props>();
const emits = defineEmits<TabsRootEmits>();

const forwarded = useForwardPropsEmits(props, emits);
</script>

<template>
  <TabsRoot v-bind="forwarded">
    <slot />
  </TabsRoot>
</template>
