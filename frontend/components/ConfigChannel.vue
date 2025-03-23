<template>
    <div v-if="channel" class="w-full max-w-[800px]">
        <h2 class="pt-3 text-3xl">{{ t('config.channelConf') }} ({{ channel.id }})</h2>
        <div class="w-full flex justify-end my-4">
            <button v-if="authStore.role === 'global_admin'" class="btn btn-sm btn-primary" @click="newChannel()">
                {{ t('config.addChannel') }}
            </button>
        </div>
        <div class="w-full">
            <label class="form-control w-full">
                <div class="label">
                    <span class="label-text">{{ t('config.name') }}</span>
                </div>
                <input
                    v-model="channel.name"
                    type="text"
                    name="name"
                    placeholder="Type here"
                    class="input input-bordered w-full !bg-base-100"
                    @keyup="isChanged"
                    :disabled="authStore.role === 'user'"
                />
            </label>

            <label class="form-control w-full mt-5">
                <div class="label">
                    <span class="label-text">{{ t('config.previewUrl') }}</span>
                </div>
                <input
                    v-model="channel.preview_url"
                    type="text"
                    name="preview_url"
                    class="input input-bordered w-full !bg-base-100"
                    @keyup="isChanged"
                    :disabled="authStore.role === 'user'"
                />
            </label>

            <label class="form-control w-full mt-5">
                <div class="label">
                    <span class="label-text">{{ t('config.extensions') }}</span>
                </div>
                <input
                    v-model="channel.extra_extensions"
                    type="text"
                    name="extra_extensions"
                    class="input input-bordered w-full !bg-base-100"
                    @keyup="isChanged"
                    :disabled="authStore.role === 'user'"
                />
            </label>

            <template v-if="authStore.role === 'global_admin'">
                <div class="mt-7 font-bold h-3">
                    <p v-if="configStore.playout.storage.shared_storage">
                        <SvgIcon name="warning" classes="inline mr-2" />
                        <span>{{ t('config.sharedStorage') }}</span>
                    </p>
                </div>
                <label class="form-control w-full mt-3">
                    <div class="label">
                        <span class="label-text">{{ t('config.publicPath') }}</span>
                    </div>
                    <input
                        v-model="channel.public"
                        type="text"
                        name="public"
                        class="input input-bordered w-full"
                        @keyup="isChanged"
                    />
                </label>

                <label class="form-control w-full mt-5">
                    <div class="label">
                        <span class="label-text">{{ t('config.playlistPath') }}</span>
                    </div>
                    <input
                        v-model="channel.playlists"
                        type="text"
                        name="playlists"
                        class="input input-bordered w-full"
                        @keyup="isChanged"
                    />
                </label>

                <div class="space-y-6 mt-8">
                    <div class="flex flex-col gap-2">
                        <label for="storage_type" class="text-sm font-medium">
                            {{ t('config.storageType') }}
                        </label>
                        <select
                            v-model="storage_type"
                            id="storage_type"
                            class="w-1/4 max-w-xs rounded-lg border border-gray-600 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-base-100"
                            @change="isChanged"
                        >
                            <option value="Local">Local</option>
                            <option value="S3">S3</option>
                        </select>
                    </div>

                    <div v-if="storage_type === 'Local'" class="flex flex-col gap-2 ml-5">
                        <label for="local_path" class="text-sm font-medium text-gray-400">
                            {{ t('config.localPath') }}
                        </label>
                        <input
                            v-model="local_path"
                            id="local_path"
                            placeholder="Enter Local Path"
                            class="w-full rounded-lg border border-gray-600 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-base-100"
                            @keyup="isChanged"
                        />
                    </div>

                    <div v-else-if="storage_type === 'S3'" class="space-y-4 ml-5">
                        <div class="flex flex-col gap-2">
                            <label for="s3_bucket" class="text-sm font-medium text-gray-400">
                                {{ t('config.s3Bucket') }}
                            </label>
                            <input
                                v-model="s3_bucket"
                                id="s3_bucket"
                                placeholder="Enter Bucket"
                                class="w-1/2 rounded-lg border border-gray-600 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-base-100"
                                @keyup="isChanged"
                            />
                        </div>

                        <div class="flex flex-col gap-2">
                            <label for="s3_endpoint" class="text-sm font-medium text-gray-400">
                                {{ t('config.s3Endpoint') }}
                            </label>
                            <input
                                v-model="s3_endpoint"
                                id="s3_endpoint"
                                placeholder="Enter Endpoint-Url:Port"
                                class="w-1/2 rounded-lg border border-gray-600 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-base-100"
                                @keyup="isChanged"
                            />
                        </div>

                        <div class="flex flex-col gap-2">
                            <label for="s3_accesskey" class="text-sm font-medium text-gray-400">
                                {{ t('config.s3AccessKey') }}
                            </label>
                            <input
                                v-model="s3_accesskey"
                                id="s3_accesskey"
                                placeholder="Enter Access Key"
                                class="w-1/2 rounded-lg border border-gray-600 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-base-100"
                                @keyup="isChanged"
                            />
                        </div>

                        <div class="flex flex-col gap-2">
                            <label for="s3_secretkey" class="text-sm font-medium text-gray-400">
                                {{ t('config.s3SecretKey') }}
                            </label>
                            <input
                                v-model="s3_secretkey"
                                id="s3_secretkey"
                                placeholder="Enter Secret Key"
                                class="w-1/2 rounded-lg border border-gray-600 px-3 py-2 text-sm focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500 bg-base-100"
                                @keyup="isChanged"
                            />
                        </div>
                    </div>
                </div>
                <label class="form-control w-full mt-8">
                    <div class="label">
                        <span class="label-text">{{ t('config.advendorEndpoint') }}</span>
                    </div>
                    <input
                        v-model="channel.advendor_endpoint"
                        type="text"
                        name="advendor_endpoint"
                        class="input input-bordered w-full"
                        @keyup="isChanged"
                    />
                </label>

                <label class="form-control w-full mt-6">
                    <div class="label">
                        <span class="label-text">{{ t('config.timezone') }}</span>
                    </div>
                    <select
                        v-model="channel.timezone"
                        class="select select-md select-bordered w-full max-w-xs"
                        @change="isChanged"
                    >
                        <option v-for="zone in Intl.supportedValuesOf('timeZone')" :key="zone" :value="zone">
                            {{ zone }}
                        </option>
                    </select>
                </label>
            </template>

            <div v-if="authStore.role !== 'user'" class="my-5 flex gap-1">
                <button class="btn" :class="saved ? 'btn-primary' : 'btn-error'" @click="addUpdateChannel()">
                    {{ t('config.save') }}
                </button>
                <button
                    v-if="
                        authStore.role === 'global_admin' && configStore.channels.length > 1 && channel.id > 1 && saved
                    "
                    class="btn btn-primary"
                    @click="deleteChannel()"
                >
                    {{ t('config.delete') }}
                </button>
                <button v-if="!saved" class="btn btn-primary text-xl" @click="resetChannel()">
                    <i class="bi-arrow-repeat" />
                </button>
            </div>
        </div>
        <GenericModal
            :title="t('config.restartTile')"
            :text="t('config.restartText')"
            :show="configStore.showRestartModal"
            :modal-action="configStore.restart"
        />
    </div>
</template>

<script setup lang="ts">
import dayjs from 'dayjs'
import { cloneDeep, isEqual } from 'lodash-es'

const S3_INDICATOR = 's3://'

const storage_type = ref('Local')
const local_path = ref('')
const s3_bucket = ref('')
const s3_endpoint = ref('')
const s3_accesskey = ref('')
const s3_secretkey = ref('')

const { t } = useI18n()

const authStore = useAuth()
const configStore = useConfig()
const mediaStore = useMedia()
const indexStore = useIndex()
const { i } = storeToRefs(useConfig())

const saved = ref(true)
const channel = ref({} as Channel)
const channelOrig = ref({} as Channel)


onMounted(() => {
    channel.value = cloneDeep(configStore.channels[i.value])
    channelOrig.value = cloneDeep(configStore.channels[i.value])

    // Set storage type based on existing data
    if (channel.value.storage.startsWith(S3_INDICATOR)) {
        storage_type.value = 'S3'
        const [, bucket, endpoint, accessKey, secretKey] =
            channel.value.storage.match(/^s3:\/\/([^\/]+)\/:([^\/]+)\/:([^\/]+)\/:([^\/]+)$/) || []
        s3_bucket.value = bucket || ''
        s3_endpoint.value = endpoint || ''
        s3_accesskey.value = accessKey || ''
        s3_secretkey.value = secretKey || ''
    } else {
        storage_type.value = 'Local'
        local_path.value = channel.value.storage || ''
    }
})

watch([i], () => {
    if (configStore.channels[i.value]) {
        channel.value = cloneDeep(configStore.channels[i.value])
    }
})

watch(
    [storage_type, s3_bucket, s3_endpoint, s3_accesskey, s3_secretkey, local_path],
    ([newStorageType, newBucket, newEndpoint, newAccessKey, newSecretKey, newLocalPath]) => {
        if (newStorageType === 'S3') {
            channel.value.storage = `${S3_INDICATOR}${newBucket}/:${newEndpoint}/:${newAccessKey}/:${newSecretKey}`
        } else {
            channel.value.storage = newLocalPath
        }
    }
)

function isChanged() {
    if (isEqual(channel.value, channelOrig.value)) {
        saved.value = true
    } else {
        saved.value = false
    }
}

function rmId(path: string) {
    return path.replace(/\/\d+$/, '')
}

function newChannel() {
    channel.value.id = configStore.channels.length + 1
    channel.value.name = `Channel ${channel.value.id}`
    channel.value.preview_url = `${window.location.protocol}//${window.location.host}/${channel.value.id}/live/stream.m3u8`
    channel.value.public = `${rmId(channel.value.public)}/${channel.value.id}`
    channel.value.playlists = `${rmId(channel.value.playlists)}/${channel.value.id}`
    channel.value.advendor_endpoint = `${rmId(channel.value.advendor_endpoint)}`
    channel.value.timezone = dayjs.tz.guess()
    channel.value.storage = ''
    if (storage_type.value === 'S3') {
        s3_bucket.value = ''
        s3_endpoint.value = ''
        s3_accesskey.value = ''
        s3_secretkey.value = ''

        storage_type.value = 'Local'
    } else if (storage_type.value === 'Local') {
        local_path.value = ''
    }

    saved.value = false
}

async function addNewChannel() {
    await $fetch('/api/channel/', {
        method: 'POST',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify(channel.value),
    })
        .then((chl) => {
            i.value = channel.value.id - 1
            configStore.channels.push(cloneDeep(chl as Channel))
            configStore.channelsRaw.push(chl as Channel)
            configStore.configCount = configStore.channels.length
            configStore.timezone = channel.value.timezone || 'UTC'

            indexStore.msgAlert('success', t('config.updateChannelSuccess'), 2)
        })
        .catch(() => {
            indexStore.msgAlert('error', t('config.updateChannelFailed'), 3)
        })
}

async function updateChannel() {
    await fetch(`/api/channel/${channel.value.id}`, {
        method: 'PATCH',
        headers: { ...configStore.contentType, ...authStore.authHeader },
        body: JSON.stringify(channel.value),
    })
        .then(() => {
            const oldTimezone = configStore.timezone
            const currentTimezone = channel.value.timezone

            for (let i = 0; i < configStore.channels.length; i++) {
                if (configStore.channels[i].id === channel.value.id) {
                    configStore.channels[i] = cloneDeep(channel.value)
                    configStore.timezone = channel.value.timezone || 'UTC'
                    break
                }
            }

            for (let i = 0; i < configStore.channelsRaw.length; i++) {
                if (configStore.channelsRaw[i].id === channel.value.id) {
                    configStore.channelsRaw[i] = cloneDeep(channel.value)
                    break
                }
            }

            channel.value = cloneDeep(configStore.channels[i.value])
            channelOrig.value = cloneDeep(configStore.channels[i.value])

            if (oldTimezone !== currentTimezone) {
                configStore.showRestartModal = true
            }

            indexStore.msgAlert('success', t('config.updateChannelSuccess'), 2)
        })
        .catch(() => {
            indexStore.msgAlert('error', t('config.updateChannelFailed'), 3)
        })
}

async function addUpdateChannel() {
    /*
        Save or update channel settings.
    */
    if (!saved.value) {
        saved.value = true

        if (configStore.channels[i.value].id !== channel.value.id) {
            await addNewChannel()
        } else {
            await updateChannel()
        }

        if (authStore.role === 'global_admin') {
            await configStore.getAdvancedConfig()
        }

        await configStore.getPlayoutConfig()
        await configStore.getUserConfig()
        await mediaStore.getTree('')
    }
}

function resetChannel() {
    channel.value = cloneDeep(configStore.channels[i.value])
    if (channel.value.storage.startsWith(S3_INDICATOR)) {
        storage_type.value = 'S3'
        const [, bucket, endpoint, accessKey, secretKey] =
            channel.value.storage.match(/^s3:\/\/([^\/]+)\/:([^\/]+)\/:([^\/]+)\/:([^\/]+)$/) || []
        s3_bucket.value = bucket || ''
        s3_endpoint.value = endpoint || ''
        s3_accesskey.value = accessKey || ''
        s3_secretkey.value = secretKey || ''

        local_path.value = ''
    } else {
        storage_type.value = 'Local'
        local_path.value = channel.value.storage || ''

        s3_bucket.value = ''
        s3_endpoint.value = ''
        s3_accesskey.value = ''
        s3_secretkey.value = ''
    }

    saved.value = true
}

async function deleteChannel() {
    if (channel.value.id === 1) {
        indexStore.msgAlert('warning', t('config.errorChannelDelete'), 2)
        return
    }

    const response = await fetch(`/api/channel/${channel.value.id}`, {
        method: 'DELETE',
        headers: authStore.authHeader,
    })

    i.value = configStore.i - 1

    if (authStore.role === 'global_admin') {
        await configStore.getAdvancedConfig()
    }

    await configStore.getChannelConfig()
    await configStore.getPlayoutConfig()
    await configStore.getUserConfig()
    await mediaStore.getTree('')

    if (response.status === 200) {
        indexStore.msgAlert('success', t('config.errorChannelDelete'), 2)
    } else {
        indexStore.msgAlert('error', t('config.deleteChannelFailed'), 2)
    }
}
</script>
