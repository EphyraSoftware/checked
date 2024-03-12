<script setup lang="ts">
import {inject, ComputedRef, ref, computed} from "vue";
import {AppAgentClient} from "@holochain/client";
import {useNotificationsStore} from "../../store/notifications-store";
import {sentence} from 'txtgen';
import {VerificationKeyDist} from "./types";

const client = inject("client") as ComputedRef<AppAgentClient>;

const emit = defineEmits<{
  (e: "distributed"): void;
}>();

const notifications = useNotificationsStore();

const name = ref("");
const selected = ref("");
const proof = ref("");
const signedProof = ref<Uint8Array>();
const creating = ref(false);
const inputType = ref<"file" | "paste">("file");
const fileInputField = ref<HTMLElement | null>(null);
const textAreaInputField = ref<HTMLElement | null>(null);
const downloadLink = ref<HTMLElement | null>(null);
const proofFileInputField = ref<HTMLElement | null>(null);
const formStage = ref<"uploadKey" | "downloadProof" | "uploadSigned">("uploadKey");

const utf8Encode = new TextEncoder();

const isGpgKeyValid = computed(() => {
  return true && selected.value;
});

const handleNewKeyProvided = (key: string) => {
  selected.value = key;

  const content = sentence();
  proof.value = content;
  const data = new Blob([content], {type: "text/plain"});
  downloadLink.value?.setAttribute("href", URL.createObjectURL(data));
};

const onSigningVerificationKeySelect = async (event: Event) => {
  if (!event.target) return;

  const files = (event.target as HTMLInputElement).files;

  if (!files || files.length === 0) return;

  const file = files[0];

  const reader = new FileReader();
  reader.readAsText(file);

  reader.onload = (evt) => {
    const key = evt.target?.result;
    if (!key) return;

    handleNewKeyProvided(key as string);
  };

  reader.onerror = (evt) => {
    notifications.pushNotification({
      message: `Error reading the key file: ${evt}`,
      type: "error",
    });
  };
};

const onPublicKeyPaste = async (event: Event) => {
  if (event.type === "paste") {
    const value = (event as ClipboardEvent).clipboardData?.getData(
        "text/plain",
    );
    if (!value) return;

    handleNewKeyProvided(value);
  } else {
    if (!event.target) return;

    const value = (event.target as HTMLTextAreaElement).value;

    // Change event will fire on click away after a paste so filter that out
    if (!value || value === selected.value) return;

    handleNewKeyProvided(value);
  }
};

const onSignedProofUploaded = async (event: Event) => {
  if (!event.target) return;

  const files = (event.target as HTMLInputElement).files;

  if (!files || files.length === 0) return;

  const file = files[0];

  const reader = new FileReader();
  reader.readAsText(file);

  reader.onload = (evt) => {
    const proof = evt.target?.result;
    if (!proof) return;

    signedProof.value = utf8Encode.encode(proof as string);
  };

  reader.onerror = (evt) => {
    notifications.pushNotification({
      message: `Error reading the proof signature: ${evt}`,
      type: "error",
    });
  };
};

const resetForm = async () => {
  selected.value = "";

  if (fileInputField.value) {
    (fileInputField.value as HTMLInputElement).value = "";
  }
  if (textAreaInputField.value) {
    (textAreaInputField.value as HTMLTextAreaElement).value = "";
  }
  if (proofFileInputField.value) {
    (proofFileInputField.value as HTMLInputElement).value = "";
  }
};

const distributeSigningVerificationKey = async () => {
  if (!client.value || !selected.value || !signedProof.value) return;

  creating.value = true;

  try {
    const dist: VerificationKeyDist = {
      verification_key: selected.value,
      key_type: {"MiniSignEd25519": null},
      proof: proof.value,
      proof_signature: Array.from(signedProof.value),
      name: name.value,
    };

    await client.value.callZome({
      role_name: "trusted",
      zome_name: "signing_keys",
      fn_name: "distribute_verification_key",
      payload: dist,
    });

    resetForm();
    emit("distributed");
  } catch (e: any) {
    notifications.pushNotification({
      message: `Failed to distribute the verification key key: ${e}`,
      type: "error",
    });
  } finally {
    creating.value = false;
  }
};
</script>

<template>
  <div class="collapse bg-base-200">
    <input type="radio" name="distribute-accordion" v-model="formStage" value="uploadKey" />
    <div class="collapse-title text-xl font-medium w-full">
      Provide your signing verification key
    </div>
    <div class="collapse-content">
      <div class="w-3/4 mx-auto">
        <input type="text" placeholder="Friendly name for your key" class="input input-bordered w-full" v-model="name" minlength="3" />

        <div class="flex flex-col items-center my-5">
          <div role="tablist" class="tabs tabs-boxed w-full">
            <a
                role="tab"
                :class="{ tab: true, 'tab-active': inputType === 'file' }"
                @click="inputType = 'file'"
            >Select file</a
            >
            <a
                role="tab"
                :class="{ tab: true, 'tab-active': inputType === 'paste' }"
                @click="inputType = 'paste'"
            >Paste</a
            >
          </div>

          <div class="my-3 min-h-24">
            <input
                v-if="inputType === 'file'"
                type="file"
                accept="text/*,.pub"
                @change="onSigningVerificationKeySelect"
                ref="fileInputField"
                class="file-input file-input-bordered file-input-primary"
            />
            <textarea
                v-else
                class="textarea textarea-ghost"
                placeholder="Paste your signing verification key here"
                @change="onPublicKeyPaste"
                @paste="onPublicKeyPaste"
                ref="textAreaInputField"
            ></textarea>
          </div>
        </div>
      </div>
      <div class="flex justify-end">
        <button class="btn btn-primary" @click="formStage = 'downloadProof'">Next</button>
      </div>
    </div>
  </div>
  <div class="collapse bg-base-200">
    <input type="radio" name="distribute-accordion" v-model="formStage" value="downloadProof" :disabled="!name || name.length <= 3 && !selected" />
    <div class="collapse-title text-xl font-medium">
      Download signing proof
    </div>
    <div class="collapse-content">
      <p>To prove that you have the signing key that goes with this verification key, please download this file and sign it:</p>
      <div class="flex justify-center">
        <a download="proof.txt" type="text/plain" ref="downloadLink" class="link link-accent">proof.txt</a>
      </div>

      <div class="flex justify-end">
        <button class="btn btn-primary" @click="formStage = 'uploadSigned'">Next</button>
      </div>
    </div>
  </div>
  <div class="collapse bg-base-200">
    <input type="radio" name="distribute-accordion" v-model="formStage" value="uploadSigned" /> <!-- :disabled="!name || name.length <= 3 && !selected" -->
    <div class="collapse-title text-xl font-medium">
      Upload signed proof
    </div>
    <div class="collapse-content">
      <div class="flex justify-center">
        <input
            v-if="inputType === 'file'"
            type="file"
            accept="text/*,*.txt.minisig"
            @change="onSignedProofUploaded"
            ref="proofFileInputField"
            class="file-input file-input-bordered file-input-primary"
        />
      </div>

      <p v-if="signedProof" class="mx-3 my-5">You're all set, when you're ready to distribute your key as '{{ name }}', click the distribute button</p>

      <div class="flex justify-end my-3">
        <div class="join">
          <button
              class="btn btn-primary join-item"
              :disabled="!isGpgKeyValid || creating"
              @click="distributeSigningVerificationKey"
          >
            <span v-if="creating" class="loading loading-spinner"></span>
            <span v-else>{{
                creating ? "Creating..." : "Distribute Key"
              }}</span>
          </button>
          <button
              class="btn btn-secondary join-item"
              :disabled="!selected"
              @click="resetForm"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
