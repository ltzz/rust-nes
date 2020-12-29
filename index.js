const fileReader = file => {
  const reader = new FileReader();

  return new Promise((resolve, reject) => {
    reader.onerror = () => {
      reader.abort();
      reject('error');
    };

    reader.onload = () => {
      resolve(reader.result);
    };

    reader.readAsArrayBuffer(file);
  });
};

window.addEventListener('load', () => {
  const f = document.getElementById('rom_select');
  f.addEventListener('change', e => {
    const input = event.target;
    if(input.files.length < 1) return;
    if(!input.files[0]) return;
    const file = input.files[0];
    console.log(file)
    const reader = new FileReader();
    fileReader(file)
      .then(buf => {
        const u8array = new Uint8Array(buf);
        console.log(u8array);
        import('./pkg/index.js').then(pkg => {
          console.log(pkg.set_rom(u8array));
        });
      }).catch(reason => {
        alert(reason);
      });
  });
});

// For more comments about what's going on here, check out the `hello_world`
// example.
import('./pkg')
  .catch(console.error);
