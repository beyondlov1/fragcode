<script setup>
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api'
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { ElTable } from 'element-plus'
import { exit } from '@tauri-apps/api/process';
import { register } from '@tauri-apps/api/globalShortcut';


const input = ref('')
const tableData = ref([])
const refInput =ref()
const singleTableRef = ref()
const currentRow = ref()

oninputchange("");


function oninputchange(value){
  console.log(value)
  let keyword = null;
  var command = value.match(/(\w+)(.*)/)
  if(command != null){
    keyword = command[1]
  }else{
    keyword = "";
  }

  console.log(keyword)
  invoke('list', { name: keyword })
    .then((response) => {
      let r = JSON.parse(response)
      if(command != null && command.length > 2 && command[2].length > 0){
        let args = command[2].trim().split(/\s+/)
        for (let j = 0; j < r.length; j++) {
          var item = r[j];
          for (let i = 0; i < args.length; i++) {
            const arg = args[i];
            item["code"] = replace(item["code"], arg, i+1)
          }
        }
      }
      tableData.value = r;
      setCurrent(tableData.value[0]);
    })
}

function replace(str, arg, argi){
  return str.replace("$"+argi, arg)
}

function onenter(value){
  let command = value.match(/(\w+)\s+(\w+)\s+(.*)/)
  if(command != null && command[1] == "add"){
    invoke('add', { abbr: command[2], code: command[3] });
    invoke('list', { name: "" })
      .then((response) => {
        tableData.value = JSON.parse(response);
        input.value = "";
      })
  }else{
    let selectedRow = null;
    if(currentRow.value){
      selectedRow = currentRow.value;
    }else if (tableData.value.length > 0){
      selectedRow  = tableData.value[0]
    }
    if(selectedRow != null){
      writeText(selectedRow["code"]);
      input.value = "";
      oninputchange(input.value);
      invoke("toggle")
    }
  }
}

function onrowclick(row, column, event){
  console.log(row["code"])
  writeText(row["code"]);
  refInput.value.focus();
  refInput.value.select();
}

function remove(row){
  console.log("row"+row.id)
  invoke('remove', { id: row.id+""});
  oninputchange("");
}

const setCurrent = (row) => {
  singleTableRef.value.setCurrentRow(row)
}
const handleCurrentChange = (val) => {
  currentRow.value = val
}

function focusInput(){
  if(refInput.value != null){
    refInput.value.focus();
  }
}

document.onkeydown = function(e) {
  if (e.ctrlKey && e.key == "z") {
    input.value = ""
  }
}


register('Ctrl+Space', () => {
  invoke("toggle")
});

setInterval(focusInput, 1000);

</script>

<template>
  <el-input ref="refInput" v-model="input" @input="oninputchange" @change="onenter" placeholder="Please input" />
  <el-table :data="tableData"  :show-header=false style="width: 100%" @cell-click="onrowclick"
  	  highlight-current-row  ref="singleTableRef" 
      @current-change="handleCurrentChange">
    <el-table-column prop="abbr" label="Abbr." width="100" />
    <el-table-column prop="code" label="Code"/>
    <el-table-column width="100">
      <template v-slot:default="{row}">
        <el-link type="primary" @click="remove(row)" >
          <span>删除</span>
        </el-link>
      </template>
    </el-table-column>
  </el-table>
</template>



