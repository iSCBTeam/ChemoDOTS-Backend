#!/usr/bin/env python3

#Searching a chemical function in a given smiles molecule, then return the result

from rdkit import Chem

from rdkit.Chem.Draw import rdMolDraw2D

from rdkit.Chem.Draw import IPythonConsole

from rdkit.Chem import AllChem

import re

import moieties



#Search chemical function (returns the index where the fct is, and its name)

def searchFct(sdf):

    

    m = Chem.MolFromMolBlock(sdf)

    # B

    boronate_alkyl=Chem.MolFromSmarts('OB(O)C')
    boronate_alkyl.name=moieties.boronate_alkyl
    boronate_alkyl.exclu = []

    boronate_aryl=Chem.MolFromSmarts('OB(O)c')
    boronate_aryl.name = moieties.boronate_aryl
    boronate_aryl.exclu = []

    # N

    #primaire=Chem.MolFromSmarts('[NH2;!$(N~[C,S,P,N]=[O,S,N]);!$(N#[C,N]);!$(N=C)]')
    #primaire.name="Amine Primaire"

    #secondaire=Chem.MolFromSmarts('[NH1;!$(N~[C,S,P,N]=[O,S,N]);!$(N#[C,N]);!$(N=C)]')
    #secondaire.name="Amine secondaire"

    amine_prim_alkyl=Chem.MolFromSmarts('C[NH2]')
    amine_prim_alkyl.name = moieties.amine_prim_alkyl
    amine_prim_alkyl.exclu = []

    amine_sec_alkyl=Chem.MolFromSmarts('C[NH]C')
    amine_sec_alkyl.name = moieties.amine_sec_alkyl
    amine_sec_alkyl.exclu = []

    amine_tert_alkyl=Chem.MolFromSmarts('CN(C)C')
    amine_tert_alkyl.name = moieties.amine_tert_alkyl
    amine_tert_alkyl.exclu = []

    amine_prim_aryl=Chem.MolFromSmarts('c[NH2]')
    amine_prim_aryl.name = moieties.amine_prim_aryl
    amine_prim_aryl.exclu = []

    amine_sec_aryl=Chem.MolFromSmarts('c[NH]c')
    amine_sec_aryl.name = moieties.amine_sec_aryl
    amine_sec_aryl.exclu = []

    amine_tert_aryl=Chem.MolFromSmarts('cN(C)C')
    amine_tert_aryl.name = moieties.amine_tert_aryl
    amine_tert_aryl.exclu = []

    nitrile_alkyl=Chem.MolFromSmarts('CC#N')
    nitrile_alkyl.name = moieties.nitrile_alkyl
    nitrile_alkyl.exclu = []

    nitrile_aryl=Chem.MolFromSmarts('cC#N')
    nitrile_aryl.name = moieties.nitrile_aryl
    nitrile_aryl.exclu = []

    aziridine_alkyl=Chem.MolFromSmarts('C1NC1C')
    aziridine_alkyl.name = moieties.aziridine_alkyl
    aziridine_alkyl.exclu = [moieties.amine_sec_alkyl]

    aziridine_aryl=Chem.MolFromSmarts('C1NC1c')
    aziridine_aryl.name = moieties.aziridine_aryl
    aziridine_aryl.exclu = [moieties.amine_sec_alkyl]

    imine_alkyl=Chem.MolFromSmarts('[C;R0](=N)C')
    imine_alkyl.name = moieties.imine_alkyl
    imine_alkyl.exclu = []

    imine_aryl=Chem.MolFromSmarts('[C;R0](=N)c')
    imine_aryl.name = moieties.imine_aryl
    imine_aryl.exclu = []

    azide_alkyl=Chem.MolFromSmarts('[N-]=[N+]=NC')
    azide_alkyl.name = moieties.azide_alkyl
    azide_alkyl.exclu = []

    azide_aryl=Chem.MolFromSmarts('[N-]=[N+]=Nc')
    azide_aryl.name = moieties.azide_aryl
    azide_aryl.exclu = []

    amidine_alkyl=Chem.MolFromSmarts('C[C;R0](N)=N')
    amidine_alkyl.name = moieties.amidine_alkyl
    amidine_alkyl.exclu = [moieties.imine_alkyl,moieties.amine_prim_alkyl]

    amidine_aryl=Chem.MolFromSmarts('c[C;R0](N)=N')
    amidine_aryl.name = moieties.amidine_aryl
    amidine_aryl.exclu = [moieties.imine_aryl,moieties.amine_prim_alkyl]

    hydrazine_alkyl=Chem.MolFromSmarts('C[NH;R0][NH2;R0]')
    hydrazine_alkyl.name = moieties.hydrazine_alkyl
    hydrazine_alkyl.exclu = []

    hydrazine_aryl=Chem.MolFromSmarts('c[NH;R0][NH2;R0]')
    hydrazine_aryl.name = moieties.hydrazine_aryl
    hydrazine_aryl.exclu = []

    # O

    alcohol=Chem.MolFromSmarts('[OH;R0][#6;!$([#6]=[O,S])]')
    alcohol.name = moieties.alcohol
    alcohol.exclu = []

    alcohol_alkyl=Chem.MolFromSmarts('C[OH]')
    alcohol_alkyl.name = moieties.alcohol_alkyl
    alcohol_alkyl.exclu = [moieties.alcohol]

    alcohol_alkyl2=Chem.MolFromSmarts('[O-]C(=O)')
    alcohol_alkyl2.name = moieties.alcohol_alkyl
    alcohol_alkyl2.exclu = []

    alcohol_aryl=Chem.MolFromSmarts('c[OH]')
    alcohol_aryl.name = moieties.alcohol_aryl
    alcohol_aryl.exclu = [moieties.alcohol]

    acid_alkyl=Chem.MolFromSmarts('C[C;R0]([OH])=O')
    acid_alkyl.name = moieties.acid_alkyl
    acid_alkyl.exclu = [moieties.alcohol_alkyl]

    acid_alkyl2=Chem.MolFromSmarts('[O-]C(=O)C')
    acid_alkyl2.name = moieties.acid_alkyl
    acid_alkyl2.exclu = [moieties.alcohol_alkyl]

    acid_aryl=Chem.MolFromSmarts('c[C;R0]([OH])=O')
    acid_aryl.name = moieties.acid_aryl
    acid_aryl.exclu = [moieties.alcohol_alkyl]

    acid_aryl2=Chem.MolFromSmarts('[O-]C(=O)c')
    acid_aryl2.name = moieties.acid_aryl
    acid_aryl2.exclu = [moieties.alcohol_alkyl]

    aldehyde_alkyl=Chem.MolFromSmarts('C[CH1;R0](=O)')
    aldehyde_alkyl.name = moieties.aldehyde_alkyl
    aldehyde_alkyl.exclu = []

    aldehyde_aryl=Chem.MolFromSmarts('c[CH1;R0](=O)')
    aldehyde_aryl.name = moieties.aldehyde_aryl
    aldehyde_aryl.exclu = []

    ketone_alkyl=Chem.MolFromSmarts('C[C;R0](=O)C')
    ketone_alkyl.name = moieties.ketone_alkyl
    ketone_alkyl.exclu = []

    ketone_aryl=Chem.MolFromSmarts('c[C;R0](=O)C')
    ketone_aryl.name = moieties.ketone_aryl
    ketone_aryl.exclu = []

    ketone_diaryl=Chem.MolFromSmarts('c[C;R0](=O)c')
    ketone_diaryl.name = moieties.ketone_diaryl
    ketone_diaryl.exclu = []

    ester_alkyl=Chem.MolFromSmarts('C[C;R0](=O)OC')
    ester_alkyl.name = moieties.ester_alkyl
    ester_alkyl.exclu = [moieties.ether_alkyl]

    ester_aryl=Chem.MolFromSmarts('c[C;R0](=O)OC')
    ester_aryl.name = moieties.ester_aryl
    ester_aryl.exclu = [moieties.ether_alkyl]

    ether_alkyl=Chem.MolFromSmarts('C[O;R0]C')
    ether_alkyl.name = moieties.ether_alkyl
    ether_alkyl.exclu = []

    ether_aryl=Chem.MolFromSmarts('C[O;R0]c')
    ether_aryl.name = moieties.ether_aryl
    ether_aryl.exclu = []

    michael_acc_alkyl=Chem.MolFromSmarts('C[C;R0](=O)[C;R0]=[C;R0]')
    michael_acc_alkyl.name = moieties.michael_acc_alkyl
    michael_acc_alkyl.exclu = [moieties.ketone_alkyl,moieties.alkene_alkyl]

    michael_acc_aryl=Chem.MolFromSmarts('c[C;R0](=O)[C;R0]=[C;R0]')
    michael_acc_aryl.name = moieties.michael_acc_aryl
    michael_acc_aryl.exclu = [moieties.alkene_alkyl]

    anhydride_alkyl=Chem.MolFromSmarts('C[C;R0](=O)O[C;R0](=O)C')
    anhydride_alkyl.name = moieties.anhydride_alkyl
    anhydride_alkyl.exclu = [moieties.ester_alkyl]

    anhydride_aryl=Chem.MolFromSmarts('c[C;R0](=O)O[C;R0](=O)c')
    anhydride_aryl.name = moieties.anhydride_aryl
    anhydride_aryl.exclu = [moieties.ester_aryl]

    dicarbonyl_1_3_alkyl=Chem.MolFromSmarts('C[C;R0](=O)[CH2][C;R0](=O)C')
    dicarbonyl_1_3_alkyl.name = moieties.dicarbonyl_1_3_alkyl
    dicarbonyl_1_3_alkyl.exclu = [moieties.ketone_alkyl]

    dicarbonyl_1_3_aryl=Chem.MolFromSmarts('c[C;R0](=O)[CH2][C;R0](=O)c')
    dicarbonyl_1_3_aryl.name = moieties.dicarbonyl_1_3_aryl
    dicarbonyl_1_3_aryl.exclu = [moieties.ketone_aryl]

    dicarbonyl_1_4_alkyl=Chem.MolFromSmarts('C[C;R0](=O)[CH2][CH2][C;R0](=O)C')
    dicarbonyl_1_4_alkyl.name = moieties.dicarbonyl_1_4_alkyl
    dicarbonyl_1_4_alkyl.exclu = [moieties.ketone_alkyl]

    dicarbonyl_1_4_aryl=Chem.MolFromSmarts('c[C;R0](=O)[CH2][CH2][C;R0](=O)c')
    dicarbonyl_1_4_aryl.name = moieties.dicarbonyl_1_4_aryl
    dicarbonyl_1_4_aryl.exclu = [moieties.ketone_aryl]

    ketone_alpha_halide=Chem.MolFromSmarts('C[C;R0](=O)[CH2][Cl,Br,I]')
    ketone_alpha_halide.name = moieties.ketone_alpha_halide
    # TODO: ketone_alpha_halide.exclu = ["Alkyl halide",moieties.ketone_alkyl] Alkyl halide ???
    ketone_alpha_halide.exclu = [moieties.ketone_alkyl]

    ketone_beta_halide=Chem.MolFromSmarts('C[C;R0](=O)[CH2][CH2][Cl,Br,I]')
    ketone_beta_halide.name = moieties.ketone_beta_halide
    # TODO: ketone_beta_halide.exclu = ["Alkyl halide",moieties.ketone_alkyl] Alkyl halide ???
    ketone_beta_halide.exclu = [moieties.ketone_alkyl]

    epoxyde_alkyl=Chem.MolFromSmarts('C1OC1C')
    epoxyde_alkyl.name = moieties.epoxyde_alkyl
    epoxyde_alkyl.exclu = []

    epoxyde_aryl=Chem.MolFromSmarts('C1OC1c')
    epoxyde_aryl.name = moieties.epoxyde_aryl
    epoxyde_aryl.exclu = []

    acyl_chloride_alkyl=Chem.MolFromSmarts('C[C;R0](Cl)=O')
    acyl_chloride_alkyl.name = moieties.acyl_chloride_alkyl
    acyl_chloride_alkyl.exclu = []

    acyl_chloride_aryl=Chem.MolFromSmarts('c[C;R0](Cl)=O')
    acyl_chloride_aryl.name = moieties.acyl_chloride_aryl
    acyl_chloride_aryl.exclu = []

    # S

    thioether_alkyl=Chem.MolFromSmarts('C[S;R0]C')
    thioether_alkyl.name = moieties.thioether_alkyl
    thioether_alkyl.exclu = []

    thioether_aryl=Chem.MolFromSmarts('C[S;R0]c')
    thioether_aryl.name = moieties.thioether_aryl
    thioether_aryl.exclu = []

    thiol_alkyl=Chem.MolFromSmarts('C[SH]')
    thiol_alkyl.name = moieties.thiol_alkyl
    thiol_alkyl.exclu = []

    thiol_aryl=Chem.MolFromSmarts('c[SH]')
    thiol_aryl.name = moieties.thiol_aryl
    thiol_aryl.exclu = []


    # O & N

    amide1=Chem.MolFromSmarts('[#6][C;R0](=[OD1])[NH2]')
    amide1.name = moieties.amide1
    amide1.exclu = []

    amide2=Chem.MolFromSmarts('[#6][C;R0](=[OD1])[NH][#6]')
    amide2.name = moieties.amide2
    amide2.exclu = []

    amide_alkyl=Chem.MolFromSmarts('C[C;R0]([NH2])=O')
    amide_alkyl.name = moieties.amide_alkyl
    amide_alkyl.exclu = [moieties.amide1,moieties.amine_prim_alkyl]

    amide_aryl=Chem.MolFromSmarts('c[C;R0]([NH2])=O')
    amide_aryl.name = moieties.amide_aryl
    amide_aryl.exclu = [moieties.amide1,moieties.amine_prim_alkyl]

    isocyanate_alkyl=Chem.MolFromSmarts('CN=C=O')
    isocyanate_alkyl.name = moieties.isocyanate_alkyl
    isocyanate_alkyl.exclu = []

    isocyanate_aryl=Chem.MolFromSmarts('cN=C=O')
    isocyanate_aryl.name = moieties.isocyanate_aryl
    isocyanate_aryl.exclu = []

    nitro_alkyl=Chem.MolFromSmarts('C[N+]([O-])=O')
    nitro_alkyl.name = moieties.nitro_alkyl
    nitro_alkyl.exclu = []

    nitro_aryl=Chem.MolFromSmarts('c[N+]([O-])=O')
    nitro_aryl.name = moieties.nitro_aryl
    nitro_aryl.exclu = []

    imide_alkyl=Chem.MolFromSmarts('C[C;R0](=O)N[C;R0](=O)C')
    imide_alkyl.name = moieties.imide_alkyl
    imide_alkyl.exclu = [moieties.amide2,moieties.amine_sec_alkyl]

    imide_aryl=Chem.MolFromSmarts('c[C;R0](=O)N[C;R0](=O)c')
    imide_aryl.name = moieties.imide_aryl
    imide_aryl.exclu = [moieties.amide2,moieties.amine_sec_alkyl]

    # O & S

    thioester_alkyl=Chem.MolFromSmarts('C[C;R0](=S)OC')
    thioester_alkyl.name = moieties.thioester_alkyl
    thioester_alkyl.exclu = [moieties.ether_alkyl]

    thioester_aryl=Chem.MolFromSmarts('c[C;R0](=S)OC')
    thioester_aryl.name = moieties.thioester_aryl
    # TODO: aryl?
    thioester_aryl.exclu = [moieties.ether_alkyl]

    vinylsulfonyl_alkyl=Chem.MolFromSmarts('C[S;R0](=O)(=O)[C;R0]=[C;R0]')
    vinylsulfonyl_alkyl.name = moieties.vinylsulfonyl_alkyl
    vinylsulfonyl_alkyl.exclu = [moieties.thioether_alkyl]

    vinylsulfonyl_aryl=Chem.MolFromSmarts('c[S;R0](=O)(=O)[C;R0]=[C;R0]')
    vinylsulfonyl_aryl.name = moieties.vinylsulfonyl_aryl
    vinylsulfonyl_aryl.exclu = [moieties.thioether_aryl]

    sulfonate_ester_alkyl=Chem.MolFromSmarts('C[S;R0](=O)(=O)OC')
    sulfonate_ester_alkyl.name = moieties.sulfonate_ester_alkyl
    sulfonate_ester_alkyl.exclu = []

    sulfonate_ester_aryl=Chem.MolFromSmarts('c[S;R0](=O)(=O)Oc')
    sulfonate_ester_aryl.name = moieties.sulfonate_ester_aryl
    sulfonate_ester_aryl.exclu = []

    sulfonylhalide_alkyl=Chem.MolFromSmarts('C[S;R0](Cl)(=O)=O')
    sulfonylhalide_alkyl.name = moieties.sulfonylhalide_alkyl
    sulfonylhalide_alkyl.exclu = []

    sulfonylhalide_aryl=Chem.MolFromSmarts('c[S;R0](Cl)(=O)=O')
    sulfonylhalide_aryl.name = moieties.sulfonylhalide_aryl
    sulfonylhalide_aryl.exclu = []

    # S & N

    thioamide_alkyl1=Chem.MolFromSmarts('C[C;R0]([NH2])=S')
    thioamide_alkyl1.name = moieties.thioamide_alkyl1
    thioamide_alkyl1.exclu = [moieties.amine_prim_alkyl]

    thioamide_alkyl2=Chem.MolFromSmarts('c[C;R0]([NH2])=S')
    thioamide_alkyl2.name = moieties.thioamide_alkyl2
    thioamide_alkyl2.exclu = [moieties.amine_prim_alkyl]

    thioisocyanate_alkyl=Chem.MolFromSmarts('CN=C=S')
    thioisocyanate_alkyl.name = moieties.thioisocyanate_alkyl
    thioisocyanate_alkyl.exclu = []

    thioisocyanate_aryl=Chem.MolFromSmarts('cN=C=S')
    thioisocyanate_aryl.name = moieties.thioisocyanate_aryl
    thioisocyanate_aryl.exclu = []

    thiourea_alkyl=Chem.MolFromSmarts('[C,c]N[C;R0](=S)N')
    thiourea_alkyl.name = moieties.thiourea_alkyl
    thiourea_alkyl.exclu = [moieties.amine_prim_alkyl, moieties.amine_sec_alkyl]

    # S & N & O

    sulfonamide_alkyl=Chem.MolFromSmarts('C[S;R0]([NH2])(=O)=O')
    sulfonamide_alkyl.name = moieties.sulfonamide_alkyl
    sulfonamide_alkyl.exclu = []

    sulfonamide_aryl=Chem.MolFromSmarts('c[S;R0]([NH2])(=O)=O')
    sulfonamide_aryl.name = moieties.sulfonamide_aryl
    sulfonamide_aryl.exclu = []

    # else

    alkyne_alkyl=Chem.MolFromSmarts('CC#C')
    alkyne_alkyl.name = moieties.alkyne_alkyl
    alkyne_alkyl.exclu = []

    alkyne_aryl=Chem.MolFromSmarts('cC#C')
    alkyne_aryl.name = moieties.alkyne_aryl
    alkyne_aryl.exclu = []

    alkene_alkyl=Chem.MolFromSmarts('C[C;R0]=[C;R0]')
    alkene_alkyl.name = moieties.alkene_alkyl
    alkene_alkyl.exclu = []

    alkene_aryl=Chem.MolFromSmarts('c[C;R0]=[C;R0]')
    alkene_aryl.name = moieties.alkene_aryl
    alkene_aryl.exclu = []

    halide_alkyl_Cl=Chem.MolFromSmarts('[CH2]Cl')
    halide_alkyl_Cl.name = moieties.halide_alkyl_Cl
    halide_alkyl_Cl.exclu = []

    halide_aryl_Cl=Chem.MolFromSmarts('cCl')
    halide_aryl_Cl.name = moieties.halide_aryl_Cl
    halide_aryl_Cl.exclu = []

    halo_pyrimidine_Cl=Chem.MolFromSmarts('Clc1ncccn1')
    halo_pyrimidine_Cl.name = moieties.halo_pyrimidine_Cl
    halo_pyrimidine_Cl.exclu = [moieties.halide_aryl_Cl]

    halide_alkyl_Br=Chem.MolFromSmarts('[CH2]Br')
    halide_alkyl_Br.name = moieties.halide_alkyl_Br
    halide_alkyl_Br.exclu = []

    halide_aryl_Br=Chem.MolFromSmarts('cBr')
    halide_aryl_Br.name = moieties.halide_aryl_Br
    halide_aryl_Br.exclu = []

    halo_pyrimidine_Br=Chem.MolFromSmarts('Brc1ncccn1')
    halo_pyrimidine_Br.name = moieties.halo_pyrimidine_Br
    halo_pyrimidine_Br.exclu = [moieties.halide_aryl_Br]

    halide_alkyl_I=Chem.MolFromSmarts('[CH2]I')
    halide_alkyl_I.name = moieties.halide_alkyl_I
    halide_alkyl_I.exclu = []

    halide_aryl_I=Chem.MolFromSmarts('cI')
    halide_aryl_I.name = moieties.halide_aryl_I
    halide_aryl_I.exclu = []

    halo_pyrimidine_I=Chem.MolFromSmarts('Ic1ncccn1')
    halo_pyrimidine_I.name = moieties.halo_pyrimidine_I
    halo_pyrimidine_I.exclu = [moieties.halide_aryl_I]

    fctB = [boronate_alkyl, boronate_aryl]

    fctN = [imine_alkyl, imine_aryl, amidine_alkyl, amidine_aryl, hydrazine_alkyl, hydrazine_aryl, aziridine_alkyl, aziridine_aryl, nitrile_alkyl, nitrile_aryl, azide_alkyl, azide_aryl, amine_prim_alkyl, amine_prim_aryl, amine_sec_alkyl, amine_sec_aryl, amine_tert_alkyl, amine_tert_aryl]

    fctO = [dicarbonyl_1_3_alkyl, dicarbonyl_1_3_aryl, dicarbonyl_1_4_alkyl, dicarbonyl_1_4_aryl, anhydride_alkyl, anhydride_aryl, ether_alkyl, ether_aryl, ester_alkyl, ester_aryl, ketone_alkyl, ketone_aryl, ketone_diaryl, michael_acc_alkyl, michael_acc_aryl, aldehyde_alkyl, aldehyde_aryl, acid_alkyl, acid_aryl, alcohol_alkyl, alcohol_aryl, alcohol, ketone_alpha_halide, ketone_beta_halide, epoxyde_alkyl, epoxyde_aryl, acyl_chloride_alkyl, acyl_chloride_aryl, alcohol_alkyl2, acid_alkyl2 , acid_aryl2]

    fctS = [thiol_alkyl, thiol_aryl, thioether_alkyl, thioether_aryl]

    fctON = [imide_alkyl, imide_aryl, amide1, amide2, amide_alkyl, amide_aryl, isocyanate_alkyl, isocyanate_aryl, nitro_alkyl, nitro_aryl, sulfonamide_alkyl, sulfonamide_aryl]

    fctOS = [thioester_alkyl, thioester_aryl, vinylsulfonyl_alkyl, vinylsulfonyl_aryl, sulfonate_ester_alkyl, sulfonate_ester_aryl, sulfonylhalide_alkyl, sulfonylhalide_aryl,  sulfonamide_alkyl, sulfonamide_aryl]

    fctSN = [thioamide_alkyl1, thioamide_alkyl2, thioisocyanate_alkyl, thioisocyanate_aryl, thiourea_alkyl, sulfonamide_alkyl, sulfonamide_aryl]

    fctElse = [halide_alkyl_Cl, halide_aryl_Cl, halo_pyrimidine_Cl, alkyne_alkyl, alkyne_aryl, alkene_alkyl, alkene_aryl,halide_alkyl_Br, halide_aryl_Br, halo_pyrimidine_Br,halide_alkyl_I, halide_aryl_I, halo_pyrimidine_I]

    dico ={}
    exclu=[]
    posex=[]




    for p in range(len(smiles)):

     

      #if B is present

      if (smiles[p] == 'B') & (p<len(smiles)):

          

          if smiles[p+1] != 'r':

                

              for i in range(len(fctB)):

                  t=m.GetSubstructMatches(fctB[i])

                  if (len(t) != 0):
                    dico[fctB[i].name]=t
                    if len(fctB[i].exclu)!=0:
                        exclu.extend(fctB[i].exclu)
                        pos=addpos(t)
                        posex.extend(pos)


                   

           

       

      #if N is present

      if (smiles[p] == 'N') & (p<len(smiles)):

                

              for i in range(len(fctN)):

                  t=m.GetSubstructMatches(fctN[i])

                  if (len(t) != 0):
                    dico[fctN[i].name]=t
                    if len(fctN[i].exclu)!=0:
                        exclu.extend(fctN[i].exclu)
                        pos=addpos(t)
                        posex.extend(pos)

                    

                    

                    

                                

                   

       

            

      #if O is present
      if (smiles[p] == 'O') & (p<len(smiles)):

                

              for i in range(len(fctO)):
                t=m.GetSubstructMatches(fctO[i])
                if (len(t) != 0):
                    dico[fctO[i].name]=t
                    if len(fctO[i].exclu)!=0:
                        exclu.extend(fctO[i].exclu)
                        pos=addpos(t)
                        posex.extend(pos)


                   

             

                

      #if S is present

      if (smiles[p] == 'S') & (p<len(smiles)):

                

              for i in range(len(fctS)):

                  t=m.GetSubstructMatches(fctS[i])
                  if (len(t) != 0):
                    dico[fctS[i].name]=t
                    if len(fctS[i].exclu)!=0:
                        exclu.extend(fctS[i].exclu)
                        pos=addpos(t)
                        posex.extend(pos)

        

        

        

      #if O & N are present

      if (smiles[p] == 'O') & (p<len(smiles)):

          

          for j in range(len(smiles)):

                if (smiles[j] == 'N'):

                

                    for k in range(len(fctON)):

                        t=m.GetSubstructMatches(fctON[k])

                        if (len(t) != 0):
                            dico[fctON[k].name]=t
                            if len(fctON[k].exclu)!=0:
                                exclu.extend(fctON[k].exclu)
                                pos=addpos(t)
                                posex.extend(pos)

     

        

     #if O & S are present

      if (smiles[p] == 'O') & (p<len(smiles)):

          

          for j in range(len(smiles)):

                if (smiles[j] == 'S'):

                

                    for k in range(len(fctOS)):

                        t=m.GetSubstructMatches(fctOS[k])

                        if (len(t) != 0):
                            dico[fctOS[k].name]=t
                            if len(fctOS[k].exclu)!=0:
                                exclu.extend(fctOS[k].exclu)
                                pos=addpos(t)
                                posex.extend(pos)

     

        

     

      #if N & S are present

      if (smiles[p] == 'N') & (p<len(smiles)):

          

          for j in range (len(smiles)):

                if (smiles[j] == 'S'):

                

                    for k in range(len(fctSN)):

                        t=m.GetSubstructMatches(fctSN[k])

                        if (len(t) != 0):
                            dico[fctSN[k].name]=t
                            if len(fctSN[k].exclu)!=0:
                                exclu.extend(fctSN[k].exclu)
                                pos=addpos(t)
                                posex.extend(pos)

      

        

      

      #Halogens and no B/N/O/S atoms

      else:

         for i in range(len(fctElse)):

             t=m.GetSubstructMatches(fctElse[i])

             if (len(t) != 0):
                dico[fctElse[i].name]=t
                if len(fctElse[i].exclu)!=0:
                    exclu.extend(fctElse[i].exclu)
                    pos=addpos(t)
                    posex.extend(pos)


          

    

    for v in dico:
        pos=str(dico[v])
        pos_tmp=addpos(dico[v])
        trouve=False
        i=0
        h=0
        cpt=0
        compt=0
        pos_finale=""
        while (not trouve) & (i<len(pos)-1):
            if pos_tmp[h] in posex:
                trouve=True
            if(pos[i]==")") & ((pos[i+1]==",")|(i==len(pos)-2)) & (not trouve):
                while (compt<=i+1):
                    pos_finale+=pos[compt]
                    compt+=1
                trouve=False
                cpt+=1
            elif(pos[i]==","):
                h+=1
            i+=1
        if (v not in exclu):
            print(v)
            print(dico[v])
        elif(compt>0):
            print(v)
            print(pos_finale)





def addpos(posi):
    t=str(posi)
    t=t.split(",")
    posex=[]
    for pos in t:
        try:
            x=re.findall('\d+', pos)[0]
            posex.append(x)
        except:
            break
    return posex
import sys

if __name__== '__main__':
   smiles=sys.argv[1]
   searchFct(smiles)

