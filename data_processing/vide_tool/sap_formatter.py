import pandas as pd
from numpy import int64

pd.set_option("display.max_columns", None)

dict_my_pr = {
    "loekz": "Deletion Indicator",
    "belnr": "Number of Material Document",
    "banfn": "Purchase Requisition",
    "bnfpo": "Item of requisition",
    "txz01": "Short Text",
    "ernam": "Created By",
    "menge": "Quantity requested",
    "afnam": "Requisitioner",
    "ebeln": "Purchase order",
    "statu": "Processing status",
    "blckd": "Blocking Indicator",
    "blckt": "Blocking Text",
    "aufnr": "Order",
    "ps_psp_pnr": "Short WBS",
    "pspnr": "Short WBS",
    "posid": "WBS Element",
    "banpr": "Purchase requisition processing state",
    "bname": "Full Name",
    "name_text": "User Name",
    "yybuyer": "Buyer",
    "ebelp": "Item",
    "matnr": "Material",
    "lfdat": "Delivery date",
    "frgdt": "Release Date",
    "rlwrt": "Total val. upon release",
    "bedat": "Purchase Order Date",
    "waers": "Currency",
    "preis": "Price unit",
    "werks": "Plant",
    "lpein": "Deliv. date category",
    "kostl": "Cost Center",
    "etenr": "Schedule Line",
    "eindt": "Delivery date",
    "cpudt": "Posting Date",
    "bldat": "Entry Date",
    "bwart": "Movement type",
    "bukrs": "Company Code",
    "anfnr": "RFQ",
    "vgabe": "Event",
    "frgzu": "Release State",
    "frgrl": "Release Completely Effected",
    "ktext": "Description",
    "objnr": "Object Number",
    "parnr": "Partner ID",
    "parvw": "Partner role",
    "counter": "Counter"
}

columns_my_pr = [
    "Purchase Requisition_EBAN",
    "Item of requisition_EBAN",
    "Material_EBAN",
    "Short Text_EBAN",
    "Short Text_EKPO",
    "Purchase order_EKET",
    "Purchase order_EKPO",
    "Item_EKPO",
    "Number of Material Document_EKBE",
    "Requisitioner_full_name_EBAN",
    "Requisitioner_EBAN",
    "Order_EBKN",
    "Order_AUFK",
    "Description_AUFK",
    "WBS FORMATED_WBS",
    "WBS Element_WBS",
    "Plant_EBAN",
    "Delivery date_EKET",
    "Delivery date_EBAN",
    "Posting Date_EKBE",
    "Event_EKBE",
    "Blocking Text_EBAN",
    "Blocking Indicator_EBAN",
    "Processing status_EBAN",
    "Purchase requisition processing state_EBAN",
    "Movement type_EKBE",
    "RFQ_EKPO",
    "Purchase Order Date_EKKO",
    "UNIQUE_KEY_EBAN",
    "Quantity requested_EBAN",
    "Buyer_full_name_EBAN",
    "Buyer_full_name_EKKO_BUYER",
    "Partner ID_full_name_IHPA",
    "Buyer_EKKO_BUYER",
    "Buyer_EBAN",
    "Purchase order_EKKO",
    "Release State_EKKO",
    "Release Completely Effected_EKKO",
    "UNIQUE_KEY_EBAN_FORMATED",
    "Cost Center_EKKN",
    "Partner ID_IHPA",
    "Currency_EBAN",
    "Price unit_EBAN"
]

# EBAN TABLE
def format_eban(df_eban):
    # Renames columns in the dataframe
    df_eban.columns = [
        dict_my_pr[col] + "_EBAN" if col in dict_my_pr else col + "_EBAN"
        for col in df_eban.columns
    ]

    # Format purchase order and item requisition from float (.0) to int
    df_eban["Purchase order_EBAN"] = df_eban["Purchase order_EBAN"].astype(str)
    df_eban["Item of requisition_EBAN"] = df_eban["Item of requisition_EBAN"].replace("", "nan").replace(".0", "")

    # Format some columns to get cleaned data
    df_eban["Item_EBAN"] = df_eban["Item_EBAN"].astype(int).astype(str)
    df_eban["Material_EBAN"] = df_eban["Material_EBAN"].astype(int).astype(str)
    df_eban["Purchase Requisition_EBAN"] = df_eban["Purchase Requisition_EBAN"].astype(int).astype(str)
    df_eban["Item of requisition_EBAN"] = df_eban["Item of requisition_EBAN"].astype(float).astype(int).astype(str)

    # Build a unique for each PO
    df_eban["UNIQUE_PO_KEY_EBAN"] = df_eban["Purchase order_EBAN"] + df_eban["Item_EBAN"]
    df_eban["UNIQUE_KEY_EBAN"] = df_eban["Purchase Requisition_EBAN"] + df_eban["Item of requisition_EBAN"]
    df_eban["UNIQUE_KEY_EBAN_FORMATED"] = df_eban["Purchase Requisition_EBAN"] + "-" + df_eban["Item of requisition_EBAN"]
    
    df_eban = df_eban.drop_duplicates()
    
    return df_eban[df_eban["Deletion Indicator_EBAN"] == ""]

# EKET TABLE
def format_eket(df_eket):
    # Renames columns in the dataframe
    df_eket.columns = [
        dict_my_pr[col] + "_EKET" if col in dict_my_pr else col + "_EKET"
        for col in df_eket.columns
    ]

    # Filter purchase requisition
    df_eket = df_eket[df_eket["Purchase Requisition_EKET"] != "MECHOB"]
    df_eket = df_eket[df_eket["Purchase Requisition_EKET"] != ""]

    # Format some columns to get cleaned data
    df_eket["Purchase Requisition_EKET"] = df_eket["Purchase Requisition_EKET"].astype(float).astype(int).astype(str) # float -> int : because too large by default
    df_eket["Item_EKET"] = pd.to_numeric(df_eket['Item_EKET'], errors='coerce')
    df_eket["Item_EKET"] = df_eket["Item_EKET"].fillna("")
    df_eket["Item_EKET"] = df_eket["Item_EKET"].astype(str)
    df_eket["Item of requisition_EKET"] = df_eket["Item of requisition_EKET"].astype(int).astype(str)
    df_eket["Schedule Line_EKET"] = df_eket["Schedule Line_EKET"].astype(int).astype(str)
    df_eket["Quantity requested_EKET"] = df_eket["Quantity requested_EKET"].astype(float).astype(str)

    # Build unique key id
    df_eket["UNIQUE_KEY_EKET"] = df_eket["Purchase Requisition_EKET"] + df_eket["Item of requisition_EKET"]
    df_eket["UNIQUE_PO_KEY_EKET"] = df_eket["Purchase order_EKET"] + df_eket["Item_EKET"]
    
    # Remove duplicated rows based on Purchase Requisition_EKET and Item of requisition_EKET
    df_eket = df_eket.sort_values(
        by=[
            "Purchase Requisition_EKET",
            "Item of requisition_EKET",
            "Purchase order_EKET",
        ]
    )

    return df_eket

# EKKO TABLE
def format_ekko(df_ekko):
    # Renames columns in the dataframe
    df_ekko.columns = [
        dict_my_pr[col] + "_EKKO" if col in dict_my_pr else col + "_EKKO"
        for col in df_ekko.columns
    ]

    # Format some columns to get cleaned data
    df_ekko["Purchase order_EKKO"] = df_ekko["Purchase order_EKKO"].astype(str)
    df_ekko = df_ekko.drop_duplicates("Purchase order_EKKO")

    return df_ekko

# EKPO TABLE
def format_ekpo(df_ekpo):
    # Renames columns in the dataframe
    df_ekpo.columns = [
        dict_my_pr[col] + "_EKPO" if col in dict_my_pr else col + "_EKPO"
        for col in df_ekpo.columns
    ]

    # Format some columns to get cleaned data
    df_ekpo["Purchase order_EKPO"] = df_ekpo["Purchase order_EKPO"].astype(str)
    df_ekpo["Item_EKPO"] = df_ekpo["Item_EKPO"].astype(int).astype(str)
    df_ekpo["Item of requisition_EKPO"] = df_ekpo["Item of requisition_EKPO"].astype(int).astype(str)

    df_ekpo["Material_EKPO"] = pd.to_numeric(df_ekpo['Material_EKPO'], errors='coerce')
    df_ekpo["Material_EKPO"] = df_ekpo["Material_EKPO"].fillna("")
    df_ekpo["Material_EKPO"] = df_ekpo["Material_EKPO"].astype(str)

    # Build a column containing the unique key of the PO for the EKPO table
    df_ekpo["UNIQUE_KEY_EKPO"] = df_ekpo["Purchase order_EKPO"] + df_ekpo["Item_EKPO"]

    return df_ekpo

# EKBE
def format_ekbe(df_ekbe):
    # Renames columns in the dataframe
    df_ekbe.columns = [
        dict_my_pr[col] + "_EKBE" if col in dict_my_pr else col + "_EKBE"
        for col in df_ekbe.columns
    ]

    # Format some columns to get cleaned data
    df_ekbe["Purchase order_EKBE"] = df_ekbe["Purchase order_EKBE"].astype(str)
    df_ekbe["Item_EKBE"] = df_ekbe["Item_EKBE"].astype(int).astype(str)

    df_ekbe["Number of Material Document_EKBE"] = df_ekbe["Number of Material Document_EKBE"].astype(float).astype(str)

    df_ekbe["Material_EKBE"] = pd.to_numeric(df_ekbe["Material_EKBE"], errors='coerce')
    df_ekbe["Material_EKBE"] = df_ekbe["Material_EKBE"].astype(str)

    df_ekbe["Movement type_EKBE"] = df_ekbe["Movement type_EKBE"].astype(str).replace("", "0")
    df_ekbe["Movement type_EKBE"] = df_ekbe["Movement type_EKBE"].astype(int)

    # Build a column containing the unique key of the PO for the EKBE table
    df_ekbe["UNIQUE_KEY_EKBE"] = df_ekbe["Purchase order_EKBE"] + df_ekbe["Item_EKBE"]

    # Filter on the Number of material document and the movement type
    df_ekbe = df_ekbe[df_ekbe["Number of Material Document_EKBE"].str.startswith("50") | df_ekbe["Number of Material Document_EKBE"].str.startswith("8")]
    df_ekbe = df_ekbe[df_ekbe["Movement type_EKBE"] < 200]

    # Replaces na by 0 for sorting purpose
    df_ekbe["Number of Material Document_EKBE"] = df_ekbe["Number of Material Document_EKBE"].astype(float)

    df_ekbe["Movement type_EKBE"] = df_ekbe["Movement type_EKBE"].astype(str).replace("0", "")

    # Remove duplicated rows based on Purchase order_EKBE and Item_EKBE
    df_ekbe = df_ekbe.sort_values(
        by=["Purchase order_EKBE", "Item_EKBE", "Number of Material Document_EKBE", "Movement type_EKBE"], ascending=True
    ).drop_duplicates(subset=["Purchase order_EKBE", "Item_EKBE"], keep="last")

    df_ekbe["Number of Material Document_EKBE"] = df_ekbe["Number of Material Document_EKBE"].astype(str).replace("0", "")

    return df_ekbe

# EBKN and WBS data
def format_ebkn_wbs(df_ebkn, df_wbs):
    # Remove useless columns from the EBKN df
    columns_to_delete_ebkn = [col for col in df_ebkn.columns if col not in dict_my_pr]
    df_ebkn = df_ebkn.drop(labels=columns_to_delete_ebkn, axis=1)

    # For both df, renames columns 
    df_ebkn.columns = [
        dict_my_pr[col] + "_EBKN" if col in dict_my_pr else col + "_EBKN"
        for col in df_ebkn.columns
    ]

    df_wbs.columns = [
        dict_my_pr[col] + "_WBS" if col in dict_my_pr else col + "_WBS"
        for col in df_wbs.columns
    ]

    ### EBKN
    # Format some columns to get cleaned data
    df_ebkn["Purchase Requisition_EBKN"] = df_ebkn["Purchase Requisition_EBKN"].astype(int).astype(str)
    df_ebkn["Item of requisition_EBKN"] = df_ebkn["Item of requisition_EBKN"].astype(int).astype(str)
    df_ebkn["Short WBS_EBKN"] = df_ebkn["Short WBS_EBKN"].astype(str)
    df_ebkn["Order_EBKN"] = df_ebkn["Order_EBKN"].astype(str).str.lstrip("0").str.slice(0, 10)

    # Build a column containing the unique key of the PO for the EKBE table
    df_ebkn["UNIQUE_KEY_EBKN"] = df_ebkn["Purchase Requisition_EBKN"] + df_ebkn["Item of requisition_EBKN"]


    ### WBS
    df_wbs["Short WBS_WBS"] = df_wbs["Short WBS_WBS"].astype(str)
    
    ### Merge
    df_ebkn = df_ebkn.merge(
        df_wbs, left_on="Short WBS_EBKN", right_on="Short WBS_WBS", how="left"
    ).drop_duplicates()

    return df_ebkn

# AUFK TABLE - WORK ORDERS DESCRIPTION
def format_aufk(df_aufk):
    # Renames columns 
    df_aufk.columns = [
        dict_my_pr[col] + "_AUFK" if col in dict_my_pr else col + "_AUFK"
        for col in df_aufk.columns
    ]

    # Format some columns to get cleaned data 
    df_aufk["Order_AUFK"] = df_aufk["Order_AUFK"].astype(str).str.lstrip('0')

    return df_aufk

# EKKN TABLE
def format_ekkn(df_ekkn):
    # Renames columns
    df_ekkn.columns = [
        dict_my_pr[col] + "_EKKN" if col in dict_my_pr else col + "_EKKN"
        for col in df_ekkn.columns
    ]

    # Format some columns to get cleaned data 
    df_ekkn["Purchase order_EKKN"] = df_ekkn["Purchase order_EKKN"].astype(str).str.lstrip('0')
    df_ekkn["Item_EKKN"] = df_ekkn["Item_EKKN"].astype(int).astype(str)
    df_ekkn["Short WBS_EKKN"] = df_ekkn["Short WBS_EKKN"].astype(int).astype(str)

    # Build a column containing the unique key of the PO for the EKBE table
    df_ekkn["UNIQUE_KEY_EKKN"] = df_ekkn["Purchase order_EKKN"] + df_ekkn["Item_EKKN"]
    #df_ekkn = df_ekkn.assign(UNIQUE_KEY_EKKN=df_ekkn["Purchase order_EKKN"] + df_ekkn["Item_EKKN"])

    return df_ekkn

# EKKO BUYER
def format_ekko_buyer(df_ekko_buyer):
    # Renames columns
    df_ekko_buyer.columns = [
        dict_my_pr[col] + "_EKKO_BUYER" if col in dict_my_pr else col + "_EKKO_BUYER"
        for col in df_ekko_buyer.columns
    ]

    # Just remove dupliacates
    df_ekko_buyer = df_ekko_buyer.drop_duplicates()

    return df_ekko_buyer

# IHPA Table
def format_ihpa(df_ihpa):
    # Renames columns
    df_ihpa.columns = [
        dict_my_pr[col] + "_IHPA" if col in dict_my_pr else col + "_IHPA"
        for col in df_ihpa.columns
    ]

    # Format some columns to get cleaned data 
    df_ihpa["Counter_IHPA"] = df_ihpa["Counter_IHPA"].astype(int)
    df_ihpa["Partner ID_IHPA"] = df_ihpa["Partner ID_IHPA"].str.upper()

    # Only keep rows containing user responsible and iggs
    df_ihpa = df_ihpa[df_ihpa["Partner role_IHPA"] == "VU"]
    df_ihpa = df_ihpa[df_ihpa["Partner ID_IHPA"].str.startswith("J") | df_ihpa["Partner ID_IHPA"].str.startswith("L")]

    # Remove duplicated rows based on Object Number_IHPA and Counter_IHPA
    df_ihpa = df_ihpa.sort_values(by=["Object Number_IHPA", "Counter_IHPA"], ascending=True).drop_duplicates(subset=["Object Number_IHPA"], keep="last")

    return df_ihpa

# IGG Table
def format_igg(df_igg):
    # Renames columns
    df_igg.columns = [
        dict_my_pr[col] + "_IGG" if col in dict_my_pr else col + "_IGG"
        for col in df_igg.columns
    ]

    df_igg['Full Name_IGG'] = df_igg['Full Name_IGG'].str.capitalize()

    return df_igg

# Merge the data
def merge_data(df_eban, df_ekpo, df_eket, df_ekkn, 
               df_ekbe, df_ekko, df_ebkn, df_ekko_buyer, 
               df_igg, df_aufk, df_ihpa):
    # Merge EBAN and EKPO
    df_eban["UNIQUE_PO_KEY_EBAN"] = df_eban["UNIQUE_PO_KEY_EBAN"].astype(str)
    df_ekpo["UNIQUE_KEY_EKPO"] = df_ekpo["UNIQUE_KEY_EKPO"].astype(str)
    df = df_eban.merge(df_ekpo, left_on="UNIQUE_PO_KEY_EBAN", right_on="UNIQUE_KEY_EKPO", how="left")

    # Merge df and eket
    df_eket["UNIQUE_KEY_EKET"] = df_eket["UNIQUE_KEY_EKET"].astype(str)
    df["UNIQUE_KEY_EBAN"] = df["UNIQUE_KEY_EBAN"].astype(str)
    df = df.merge(df_eket, left_on="UNIQUE_PO_KEY_EBAN", right_on="UNIQUE_PO_KEY_EKET", how="left")

    # Merge df and ekkn and get only non deleted POs
    df_ekkn["UNIQUE_KEY_EKKN"] = df_ekkn["UNIQUE_KEY_EKKN"].astype(str)
    df = df.merge(df_ekkn, left_on="UNIQUE_KEY_EKPO", right_on="UNIQUE_KEY_EKKN", how="left")
    df = df[df["Deletion Indicator_EBAN"] == ""].drop_duplicates(subset=["Purchase Requisition_EBAN", "Item of requisition_EBAN"])

    # Merge df and ekbe
    df_ekbe["UNIQUE_KEY_EKBE"] = df_ekbe["UNIQUE_KEY_EKBE"].astype(str)
    df = df.merge(df_ekbe, left_on="UNIQUE_KEY_EKPO", right_on="UNIQUE_KEY_EKBE", how="left")

    # Merge df and ekko
    df_ekko["Purchase order_EKKO"] = df_ekko["Purchase order_EKKO"].astype(str)
    df["Purchase order_EKPO"] = df["Purchase order_EKPO"].astype(str)
    df = df.merge(df_ekko, left_on="Purchase order_EKPO", right_on="Purchase order_EKKO", how="left")

    # Merge df and ebkn
    df_ebkn["UNIQUE_KEY_EBKN"] = df_ebkn["UNIQUE_KEY_EBKN"].astype(str)
    df = df.merge(df_ebkn, left_on="UNIQUE_KEY_EBAN", right_on="UNIQUE_KEY_EBKN", how="left")

    # Merge df and ekko buyer
    df_ekko_buyer["Purchase order_EKKO_BUYER"] = df_ekko_buyer["Purchase order_EKKO_BUYER"].astype(str)
    df["Purchase order_EKET"] = df["Purchase order_EKET"].astype(str).replace('.0','',regex=False)
    df = df.merge(df_ekko_buyer, left_on="Purchase order_EKET", right_on="Purchase order_EKKO_BUYER", how="left")

    # Merge df and aufk
    df = df.merge(df_aufk, left_on="Order_EBKN", right_on="Order_AUFK", how="left")

    # Merge df and ihpa
    df = df.merge(df_ihpa, left_on="Object Number_AUFK", right_on="Object Number_IHPA", how="left")

    # Merge df and igg
    df['Requisitioner_EBAN'] = df['Requisitioner_EBAN'].str.capitalize()
    df['Buyer_EKKO_BUYER'] = df['Buyer_EKKO_BUYER'].str.capitalize()
    df['Buyer_EBAN'] = df['Buyer_EBAN'].str.capitalize()

    df_igg.rename(columns={"User Name_IGG": "Requisitioner_full_name_EBAN"}, inplace = True)
    df = df.merge(df_igg, left_on="Requisitioner_EBAN", right_on="Full Name_IGG", how="left")
    df = df.drop(columns=["Full Name_IGG"])

    df_igg.rename(columns={"Requisitioner_full_name_EBAN": "Buyer_full_name_EBAN"}, inplace = True)
    df = df.merge(df_igg, left_on="Buyer_EBAN", right_on="Full Name_IGG", how="left")
    df = df.drop(columns=["Full Name_IGG"])

    df_igg.rename(columns={"Buyer_full_name_EBAN": "Buyer_full_name_EKKO_BUYER"}, inplace = True)
    df = df.merge(df_igg, left_on="Buyer_EKKO_BUYER", right_on="Full Name_IGG", how="left")
    df = df.drop(columns=["Full Name_IGG"])

    df_igg.rename(columns={"Buyer_full_name_EKKO_BUYER": "Partner ID_full_name_IHPA"}, inplace = True)
    df = df.merge(df_igg, left_on="Partner ID_IHPA", right_on="Full Name_IGG", how="left")
    df = df.drop(columns=["Full Name_IGG"])

    return df

# Process the data
def process_data(df):
    # DATA TREATMENT
    df["Order_EBKN"] = df["Order_EBKN"].fillna("N/A").str.lstrip("0").str.slice(0, 10)

    # FORMATING EBAN_PURCHASE_ORDER
    res = []
    for i in df["Purchase order_EBAN"]:
        if len(str(i)) != 12 and str(i) != "nan":
            res.append(str(i) + ".0")
        else:
            res.append("N/A")
    df["Purchase order_EBAN"] = pd.Series(res)

    # FORMATING EKET_PURCHASE_ORDER
    res = []
    for i in df["Purchase order_EKET"]:
        if str(i) != "nan":
            res.append(str(i))
        else:
            res.append("N/A")
    df["Purchase order_EKET"] = pd.Series(res)

    # FORMATING WBS
    res_wbs_formated = []
    for wbs in df["WBS Element_WBS"]:
        wbs_str = str(wbs)
        if wbs_str != "nan":
            res_wbs_formated.append(
                f"{wbs_str[0:2]}-{wbs_str[2:7]}-{wbs_str[7:9]}-{wbs_str[9:15]}-{wbs_str[15:]}"
            )
        else:
            res_wbs_formated.append("N/A")
    WBS = pd.Series(res_wbs_formated)
    df["WBS FORMATED_WBS"] = WBS

    # DROPING DUPLICATES
    df = df.drop_duplicates(subset=["Purchase Requisition_EBAN", "Item of requisition_EBAN"])
    
    # Keep only needed columns
    df = df[columns_my_pr]

    # Put one column as int
    df["Item of requisition_EBAN"] = df["Item of requisition_EBAN"].astype(int)

    # Sort the data
    df = df.sort_values(by=["Purchase Requisition_EBAN", "Item of requisition_EBAN"])

    return df
