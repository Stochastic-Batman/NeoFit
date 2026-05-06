// run with: npm run codama
import { createFromRoot } from 'codama'
import { rootNodeFromAnchor } from '@codama/nodes-from-anchor'
import { renderVisitor } from '@codama/renderers-js'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

import * as typescriptPlugin from 'prettier/plugins/typescript'
import * as estreePlugin from 'prettier/plugins/estree'
import * as babelPlugin from 'prettier/plugins/babel'

const idl = JSON.parse(
  readFileSync(join(__dirname, 'target/idl/neofit.json'), 'utf8')
)

const codama = createFromRoot(rootNodeFromAnchor(idl))

codama.accept(
  renderVisitor(join(__dirname, 'app/src/lib/generated'), {
    prettierOptions: {
      useTabs: true,
      singleQuote: true,
      plugins: [
        typescriptPlugin,
        estreePlugin,
        babelPlugin,
      ],
    }
  })
)
