#
#   Convert the web page at https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html
#   into a machine readable json document. Must be downloaded from a browser AOT
#

from bs4 import BeautifulSoup
import json

json_doc = {}

def build_signature_object(sig):
    name = sig.find("span", class_="name").text
    rettype = sig.find("span", class_="rettype").text
    param_types = [x.text for x in sig.find_all("span", class_="param_type")]
    param_names = [x.text for x in sig.find_all("span", class_="param_name")]
    params = []
    if len(param_names) == len(param_types):
        for i in range(len(param_names)):
            params.append({"name": param_names[i], "type": param_types[i]})
    else:
        assert len(param_names) == 0
        assert len(param_types) == 1
        assert param_types[0] == "void"
        params.append({"name": "", "type": "void"})
    return {
        "name":  name,
        "params": params,
        "rettype": rettype
    }

def build_synopsis_object(syn):
    cpuids = syn.find_all("span", class_="cpuid")
    return {
        "cpuids": [x.text for x in cpuids]
    }

def handle_intrinsic(intrinsic):
    classes = intrinsic["class"]
    if "intrinsic" in classes:
        classes.remove("intrinsic")
        assert len(classes) == 1
        if not (classes[0] in json_doc):
            json_doc[classes[0]] = []
            #print(f"class: {classes[0]}")
        sig = intrinsic.find("div", class_="signature")
        sig = sig.find("span", class_="sig")
        details = intrinsic.find("div", class_="details")
        instruction = intrinsic.find("div", class_="instruction")
        if instruction == None:
            instruction = ""
        else:
            instruction = instruction.text
        synopsis_obj = None
        if details is None:
            synopsis_obj = {}
            desc_obj = ""
            operation_obj = ""
        else:
            syn = details.find("div", class_="synopsis")
            if syn is None:
                synopsis_obj = {}
            else:
                synopsis_obj = build_synopsis_object(syn)
                
            desc = details.find("div", class_="description")
            if desc is None:
                desc_obj = ""
            else:
                desc_obj = desc.text

            op = details.find("div", class_="operation")
            if op is None:
                operation_obj = ""
            else:
                operation_obj = op.text
        
        intrisic_obj = {
            "instruction": instruction,
            "signature": build_signature_object(sig),
            "synopsis": synopsis_obj,
            "description": desc_obj,
            "operation": operation_obj
        }
        json_doc[classes[0]].append(intrisic_obj)

def main():
    
    txt = ""
    with open("Intel® Intrinsics Guide.html", "r") as f:
        txt = f.read()
    soup = BeautifulSoup(txt, features="html.parser")
    i_list = soup.find('div', id='intrinsics_list')
    #print(f"num children {len(list(i_list.children))}")

    for child in i_list.children:
        handle_intrinsic(child)

    with open('intrinsics.json', 'w') as fp:
        json.dump(json_doc, fp, indent=2)


main()
