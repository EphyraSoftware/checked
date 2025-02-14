import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    fileParallelism: false, // run tests serially, multi-conductor tests max out the machine anyway
    testTimeout: 60*1000*5, // 5 mins
    retry: 3, // Add a retry, some of the tests are flaky but it's most likely because of Holochain and not the tests
  },
})

