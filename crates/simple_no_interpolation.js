// Ellex Runtime Helpers
const EllexRuntime = {
  tell: (value) => {
    if (typeof value === 'object' && value !== null) {
      console.log(JSON.stringify(value, null, 2));
    } else {
      console.log(String(value));
    }
  },
  ask: async (question) => {
    if (typeof process !== 'undefined' && process.stdin) {
      // Node.js environment
      const readline = require('readline');
      const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout
      });
      return new Promise(resolve => {
        rl.question(question + ' ', answer => {
          rl.close();
          resolve(answer);
        });
      });
    } else {
      // Browser environment
      return Promise.resolve(prompt(question) || '');
    }
  },
  isNumber: (value) => typeof value === 'number' && !isNaN(value),
  isString: (value) => typeof value === 'string',
  isList: (value) => Array.isArray(value),
  safeAdd: (a, b) => {
    const numA = Number(a);
    const numB = Number(b);
    if (isNaN(numA) || isNaN(numB)) throw new Error('Cannot add non-numbers');
    return numA + numB;
  },
};

async function main() {
  let variables = new Map();

  EllexRuntime.tell("Hello world");
}

// Execute program
main().catch(error => {
  console.error('Ellex Error:', error.message);
  process.exit(1);
});