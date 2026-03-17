import { ref, type Ref } from 'vue'
import { updateAccountName } from '@/services/getters'

interface AccountOption {
  id: number
  label: string
}

export function useAccountRename(accountOptions: Ref<AccountOption[]>) {
  const renameAccountId = ref<number | null>(null)
  const renameAccountName = ref('')
  const renameStatus = ref('')

  function onRenameAccountSelect() {
    const match = accountOptions.value.find(a => a.id === renameAccountId.value)
    renameAccountName.value = match ? match.label : ''
    renameStatus.value = ''
  }

  async function submitRename() {
    if (!renameAccountId.value || !renameAccountName.value.trim()) return
    try {
      await updateAccountName(renameAccountId.value, renameAccountName.value.trim())
      const match = accountOptions.value.find(a => a.id === renameAccountId.value)
      if (match) match.label = renameAccountName.value.trim()
      renameStatus.value = 'Updated!'
    } catch (e) {
      renameStatus.value = 'Error: ' + String(e)
    }
  }

  return {
    renameAccountId,
    renameAccountName,
    renameStatus,
    onRenameAccountSelect,
    submitRename,
  }
}
