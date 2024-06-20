import { defineConfig } from 'vitest/config'

export default defineConfig({
  test: {
    fileParallelism: false, // run tests serially, multi-conductor tests max out the machine anyway
    testTimeout: 60*1000*3 // 3  mins
  },
})

