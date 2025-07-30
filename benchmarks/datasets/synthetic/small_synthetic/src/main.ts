import { Module0 } from './module_0000';
import { Module1 } from './module_0001';
import { Module2 } from './module_0002';
import { Module3 } from './module_0003';
import { Module4 } from './module_0004';
import { Module5 } from './module_0005';
import { Module6 } from './module_0006';
import { Module7 } from './module_0007';
import { Module8 } from './module_0008';
import { Module9 } from './module_0009';

async function main() {
  console.log('Starting benchmark application...');

  const module0 = new Module0({
    id: 0,
    name: 'Module0',
    enabled: true
  });
  const module1 = new Module1({
    id: 1,
    name: 'Module1',
    enabled: true
  });
  const module2 = new Module2({
    id: 2,
    name: 'Module2',
    enabled: true
  });
  const module3 = new Module3({
    id: 3,
    name: 'Module3',
    enabled: true
  });
  const module4 = new Module4({
    id: 4,
    name: 'Module4',
    enabled: true
  });
  const module5 = new Module5({
    id: 5,
    name: 'Module5',
    enabled: true
  });
  const module6 = new Module6({
    id: 6,
    name: 'Module6',
    enabled: true
  });
  const module7 = new Module7({
    id: 7,
    name: 'Module7',
    enabled: true
  });
  const module8 = new Module8({
    id: 8,
    name: 'Module8',
    enabled: true
  });
  const module9 = new Module9({
    id: 9,
    name: 'Module9',
    enabled: true
  });

  console.log('All modules initialized');
}

if (require.main === module) {
  main().catch(console.error);
}