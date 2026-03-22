import { globSync } from 'glob';
import * as path from 'path';

type PushFile = {
  path: string;
  language: string;
  namespace: string;
};

type TolgeeConfig = {
  apiUrl: string;
  projectId: number;
  format: 'JSON_C';
  push: {
    files: PushFile[];
    removeOtherKeys: boolean;
  };
  pull: {
    path: string;
    fileStructureTemplate: string;
  };
};

// 1. Find all your translation files
// This looks for: ./modules/**/locales/*.json and ./executables/**/locales/*.json
const files = globSync('./**/locales/*.json');

// 2. Map these files to Tolgee's expected format
const pushFiles: PushFile[] = files.map((filePath) => {
  // Extract namespace: everything before "/locales/"
  // Example: "modules/authentication/locales/en.json" -> "modules/authentication"
  const namespace = filePath.split(`${path.sep}locales${path.sep}`)[0].replace(/^\.\//, '');
  
  console.log(`Processing file: ${filePath}: namespace: ${namespace}`);

  // Extract language: the filename without extension
  // Example: "en.json" -> "en"
  const language = path.basename(filePath, '.json');

  return {
    path: filePath,
    language,
    namespace,
  };
});

const config: TolgeeConfig = {
  apiUrl: 'https://translate.anneraud.fr',
  projectId: 2,
  format: 'JSON_C',
  push: {
    files: pushFiles, // Use the dynamically generated list
    removeOtherKeys: true,
  },
  pull: {
    path: './',
    fileStructureTemplate: '{namespace}/locales/{languageTag}.json',
  },
};

export default config;