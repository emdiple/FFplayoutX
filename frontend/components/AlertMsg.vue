<template>
    <div class="alert-container fixed top-20 right-3 z-40 flex flex-col space-y-3 items-end w-full">
        <div
            v-if="indexStore.showAlert"
            role="alert"
            class="alert w-auto min-w-64 max-w-[90%] md:max-w-[50%] xl:max-w-[40%]"
            :class="`alert-${indexStore.alertVariant}`"
        >
            <SvgIcon :name="indexStore.alertVariant" />
            {{ indexStore.alertMsg }}
        </div>
        <div
            v-if="indexStore.secShowAlert"
            role="alert"
            class="alert w-auto min-w-64 max-w-[90%] md:max-w-[50%] xl:max-w-[40%]"
            :class="`alert-${indexStore.secAlertVariant}`"
        >
            <SvgIcon :name="indexStore.secAlertVariant" />
            {{ indexStore.secAlertMsg }}
        </div>
    </div>
</template>

<script setup lang="ts">
const indexStore = useIndex()
</script>